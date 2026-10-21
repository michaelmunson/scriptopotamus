use crate::error::{CompileError, Result};
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

#[derive(Debug)]
pub struct PackageInfo {
    pub name: String,
    pub version: Version,
    pub description: String,
}

impl PackageInfo {
    pub fn parse(path: &Path, line: usize, args: &[String]) -> Result<Self> {
        let error = |message| CompileError::at(path, line, message);
        if !(2..=3).contains(&args.len()) {
            return Err(error("@info: expected <name> <version> [<description>]"));
        }
        let values = args
            .iter()
            .map(|raw| literal(raw).ok_or_else(|| error("@info: values must be literals")))
            .collect::<Result<Vec<_>>>()?;
        let name = &values[0];
        if crate::paths::validate_name(name).ok().as_deref() != Some(name.as_str()) {
            return Err(error("@info: invalid name"));
        }
        let version = Version::parse(&values[1])
            .ok_or_else(|| error("@info: expected a stable version such as 1.2.3"))?;
        Ok(Self {
            name: name.clone(),
            version,
            description: values.get(2).cloned().unwrap_or_default(),
        })
    }
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
            .all(|b| b.is_ascii_alphanumeric() || b"_.-+".contains(&b)))
    .then(|| raw.to_string())
}
