use crate::ast::{Program, Stmt};
use crate::error::{CompileError, Result};
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Version(pub [u64; 3]);

impl Version {
    pub fn parse(raw: &str) -> Option<Self> {
        let parts: Vec<_> = raw.split('.').collect();
        if parts.len() != 3 {
            return None;
        }
        let mut version = [0; 3];
        for (slot, part) in version.iter_mut().zip(parts) {
            if part.is_empty()
                || !part.bytes().all(|b| b.is_ascii_digit())
                || (part.len() > 1 && part.starts_with('0'))
            {
                return None;
            }
            *slot = part.parse().ok()?;
        }
        Some(Self(version))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FileKind {
    Script,
    Package,
}

impl FileKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Script => "script",
            Self::Package => "package",
        }
    }
}

/// File metadata is optional; installable entries additionally require a name and version.
#[derive(Clone, Debug)]
pub struct FileInfo {
    pub name: Option<String>,
    pub version: Option<Version>,
    pub description: String,
    pub author: Option<String>,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub kind: Option<FileKind>,
    pub autoload: bool,
}

impl Default for FileInfo {
    fn default() -> Self {
        Self {
            name: None,
            version: None,
            description: String::new(),
            author: None,
            license: None,
            repository: None,
            kind: None,
            autoload: true,
        }
    }
}

impl FileInfo {
    pub fn parse(path: &Path, line: usize, args: &[String]) -> Result<Self> {
        let error = |message| CompileError::at(path, line, message);
        let mut info = Self::default();
        // Keep the original package-manager syntax working.
        if args.first().is_some_and(|arg| !arg.starts_with("--")) {
            if !(2..=3).contains(&args.len()) {
                return Err(error(
                    "@info: expected <name> <version> [<description>]".into(),
                ));
            }
            info.name = Some(args[0].clone());
            info.version = Some(parse_version(path, line, &args[1])?);
            if let Some(raw) = args.get(2) {
                info.description = parse_literal(path, line, raw)?;
            }
        } else {
            let mut seen = BTreeSet::new();
            let mut i = 0;
            while i < args.len() {
                let flag = args[i].as_str();
                if !seen.insert(flag) {
                    return Err(error(format!("@info: duplicate option {flag}")));
                }
                match flag {
                    "--script" | "--package" => {
                        if info.kind.is_some() {
                            return Err(error(
                                "@info: --script and --package are mutually exclusive".into(),
                            ));
                        }
                        info.kind = Some(if flag == "--script" {
                            FileKind::Script
                        } else {
                            FileKind::Package
                        });
                    }
                    "--name" | "--description" | "--version" | "--author" | "--license"
                    | "--repository" | "--autoload" => {
                        i += 1;
                        let raw = args
                            .get(i)
                            .filter(|value| !value.starts_with("--"))
                            .ok_or_else(|| error(format!("@info: {flag} requires a value")))?;
                        match flag {
                            "--name" => info.name = Some(raw.clone()),
                            "--version" => info.version = Some(parse_version(path, line, raw)?),
                            "--description" => info.description = parse_literal(path, line, raw)?,
                            "--author" => info.author = Some(parse_literal(path, line, raw)?),
                            "--license" => info.license = Some(parse_literal(path, line, raw)?),
                            "--repository" => {
                                info.repository = Some(parse_literal(path, line, raw)?)
                            }
                            "--autoload" => {
                                info.autoload = match parse_literal(path, line, raw)?.as_str() {
                                    "enable" => true,
                                    "disable" => false,
                                    _ => {
                                        return Err(error(
                                            "@info: --autoload expects enable or disable".into(),
                                        ));
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    _ => return Err(error(format!("@info: unknown option {flag}"))),
                }
                i += 1;
            }
        }
        if let Some(raw) = &info.name {
            let name = parse_literal(path, line, raw)?;
            if crate::paths::validate_name(&name).ok().as_deref() != Some(name.as_str()) {
                return Err(CompileError::at(path, line, "@info: invalid name"));
            }
            info.name = Some(name);
        }
        Ok(info)
    }

    /// Validate before imports are merged, so each source file owns its declaration.
    pub fn from_program(path: &Path, program: &Program) -> Result<Option<Self>> {
        fn check_nested(path: &Path, body: &[Stmt]) -> Result<()> {
            for stmt in body {
                match stmt {
                    Stmt::MacroCall { name, line, .. } | Stmt::MacroDef { name, line, .. }
                        if name == "info" =>
                    {
                        return Err(CompileError::at(
                            path,
                            *line,
                            "@info must be at the top of the file",
                        ));
                    }
                    Stmt::Function { body, .. }
                    | Stmt::MacroDef { body, .. }
                    | Stmt::For { body, .. }
                    | Stmt::While { body, .. } => check_nested(path, body)?,
                    Stmt::If {
                        branches,
                        else_body,
                        ..
                    } => {
                        for branch in branches {
                            check_nested(path, &branch.body)?;
                        }
                        if let Some(body) = else_body {
                            check_nested(path, body)?;
                        }
                    }
                    Stmt::Case { arms, .. } => {
                        for arm in arms {
                            check_nested(path, &arm.body)?;
                        }
                    }
                    _ => {}
                }
            }
            Ok(())
        }
        let declarations: Vec<_> = program
            .body
            .iter()
            .enumerate()
            .filter_map(|(index, stmt)| match stmt {
                Stmt::MacroCall { name, args, line } if name == "info" => {
                    Some((index, args, *line))
                }
                _ => None,
            })
            .collect();
        if declarations.len() > 1 {
            return Err(CompileError::at(
                path,
                declarations[1].2,
                "expected exactly one top-level @info",
            ));
        }
        let info = if let Some((index, args, line)) = declarations.first() {
            if *index != 0 {
                return Err(CompileError::at(
                    path,
                    *line,
                    "@info must be at the top of the file",
                ));
            }
            Some(Self::parse(path, *line, args)?)
        } else {
            None
        };
        check_nested(path, &program.body[usize::from(info.is_some())..])?;
        Ok(info)
    }
}

#[derive(Debug)]
pub struct PackageInfo {
    pub name: String,
    pub version: Version,
    pub description: String,
    pub metadata: FileInfo,
}

impl PackageInfo {
    pub fn parse(path: &Path, line: usize, args: &[String]) -> Result<Self> {
        let error = |message| CompileError::at(path, line, message);
        let metadata = FileInfo::parse(path, line, args)?;
        let name = metadata
            .name
            .clone()
            .ok_or_else(|| error("@info: installable entries require --name"))?;
        let version = metadata
            .version
            .clone()
            .ok_or_else(|| error("@info: installable entries require --version"))?;
        Ok(Self {
            name,
            version,
            description: metadata.description.clone(),
            metadata,
        })
    }
}

fn parse_version(path: &Path, line: usize, raw: &str) -> Result<Version> {
    Version::parse(&parse_literal(path, line, raw)?).ok_or_else(|| {
        CompileError::at(path, line, "@info: expected a stable version such as 1.2.3")
    })
}

fn parse_literal(path: &Path, line: usize, raw: &str) -> Result<String> {
    literal(raw).ok_or_else(|| CompileError::at(path, line, "@info: values must be literals"))
}

fn literal(raw: &str) -> Option<String> {
    if let Some(value) = raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\'')) {
        return (!value.contains('\'')).then(|| value.to_string());
    }
    if let Some(value) = raw.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
        return (!value.contains(['"', '$', '`', '\\'])).then(|| value.to_string());
    }
    (!raw.is_empty()
        && raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.-+/:@".contains(&b)))
    .then(|| raw.to_string())
}
