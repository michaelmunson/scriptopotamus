#!/usr/bin/env bash

function package_error {
  printf 'install: %s\n' "$*" >&2
  return 1
}

function package_requested {
  local arg
  # A script's own flags belong to that script. Installer flags precede the
  # repository; once a script filename/subcommand appears, stop looking.
  for arg in "$@"; do
    case $arg in
      -i|--install) return 0 ;;
      -*) ;;
      *) return 1 ;;
    esac
  done
  return 1
}

function package_help {
  cat <<'EOF'
usage: scrippo -i (-p | -s) [-g] [--tag VERSION] <repository> [directory]

  -i, --install   Install a tagged Git repository
  -p, --package   Compile packages/*.scrippo to Bash
  -s, --script    Keep a repository's script sources
  -g, --global    Use ~/.scriptopotamus instead of directory (default: ./)
      --tag      Select a stable version tag; default: highest version

Local installs use directory/packages/<repo>/content or directory/scripts/<repo>/content.
Existing installations are never overwritten. Requires Git, jq, and Bash 4+.
EOF
}

# A subshell owns its temporary files and traps without affecting the caller.
function package_install (
  set -euo pipefail
  local mode= global=false tag= install=false claimed=false repo name root target scratch ref version file relative output entry
  local -a positional=() files=()
  while (( $# > 0 )); do
    case $1 in
      -i|--install) install=true ;;
      -p|--package)
        [[ -z $mode ]] || { package_error 'choose exactly one of --package and --script'; exit 1; }
        mode=packages ;;
      -s|--script)
        [[ -z $mode ]] || { package_error 'choose exactly one of --package and --script'; exit 1; }
        mode=scripts ;;
      -g|--global) global=true ;;
      --tag)
        (( $# >= 2 )) && [[ -n $2 ]] || { package_error '--tag requires a version'; exit 1; }
        tag=$2
        shift ;;
      -h|--help) package_help; exit 0 ;;
      --) shift; positional+=("$@"); break ;;
      -*) package_error "unknown option '$1'"; exit 1 ;;
      *) positional+=("$1") ;;
    esac
    shift
  done
  [[ $install == true && -n $mode ]] || { package_error 'expected --install and exactly one of --package or --script'; exit 1; }
  (( ${#positional[@]} >= 1 && ${#positional[@]} <= 2 )) || { package_help >&2; exit 1; }
  if [[ $global == true && ${#positional[@]} != 1 ]]; then
    package_error '--global cannot be combined with a destination directory'; exit 1
  fi
  if [[ -n $tag ]] && ! package_version_valid "${tag#v}"; then
    package_error "invalid version tag '$tag'"; exit 1
  fi
  command -v git >/dev/null || { package_error 'Git is required'; exit 1; }
  repo=${positional[0]%/}
  name=${repo##*/}
  name=${name##*:}
  name=${name%.git}
  [[ $name =~ ^[a-zA-Z0-9_][a-zA-Z0-9_.-]*$ ]] || { package_error 'repository must have a simple basename'; exit 1; }
  root=${positional[1]:-.}
  [[ $global == false ]] || root=$HOME/.scriptopotamus
  mkdir -p -- "$root/$mode"
  root=$(cd -- "$root/$mode" && pwd -P)
  target=$root/$name
  [[ ! -e $target && ! -L $target ]] || { package_error "destination already exists: $target"; exit 1; }
  scratch=$(mktemp -d "$root/.scrippo-install.XXXXXXXX")
  trap 'rm -rf -- "$scratch"; if [[ $claimed == true && ! -e $target/content ]]; then rmdir -- "$target" 2>/dev/null || true; fi' EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM
  GIT_TERMINAL_PROMPT=0 git -c core.hooksPath=/dev/null clone --quiet --no-checkout -- "$repo" "$scratch/git"
  if [[ -z $tag ]]; then
    while IFS= read -r ref; do
      if package_version_valid "${ref#v}"; then
        printf '%s\t%s\n' "${ref#v}" "$ref"
      fi
    done < <(git -C "$scratch/git" for-each-ref --format='%(refname:strip=2)' refs/tags) > "$scratch/versions"
    # Normalize v prefixes before comparing numeric version components.
    sort -t . -k1,1nr -k2,2nr -k3,3nr "$scratch/versions" > "$scratch/sorted"
    IFS=$'\t' read -r version tag < "$scratch/sorted" || true
  fi
  [[ -n $tag ]] || { package_error 'repository has no stable version tags (expected 1.2.3 or v1.2.3)'; exit 1; }
  version=${tag#v}
  ref=$(git -C "$scratch/git" rev-parse --verify "refs/tags/$tag^{commit}") || { package_error "tag not found: $tag"; exit 1; }
  # Reject links/submodules before extraction, including links in support files.
  git -C "$scratch/git" ls-tree -r "$ref" > "$scratch/tree"
  while IFS= read -r entry; do
    case $entry in
      120000*|160000*) package_error 'repository links and submodules are not supported'; exit 1 ;;
    esac
  done < "$scratch/tree"
  mkdir "$scratch/source" "$scratch/output"
  # Extract from the object tree; neither checkout filters nor hooks run.
  git -C "$scratch/git" archive "$ref" | tar -x -C "$scratch/source"
  [[ -d $scratch/source/$mode ]] || { package_error "repository has no $mode/ directory"; exit 1; }
  while IFS= read -r -d '' file; do
    files+=("$file")
  done < <(find "$scratch/source/$mode" -type f -name '*.scrippo' -print0)
  (( ${#files[@]} > 0 )) || { package_error "no .scrippo files in $mode/"; exit 1; }
  for file in "${files[@]}"; do
    package_validate_info "$file" "$version"
    if [[ $mode == packages ]]; then
      relative=${file#"$scratch/source/packages/"}
      output=$scratch/output/${relative%.scrippo}.bash
      mkdir -p -- "$(dirname "$output")"
      # The compiler emits code without running it. Imports are confined to
      # this link-free source tree, and compile state is isolated per file.
      (set +eu; unset SCRIPPO_IMPORT_DIR SCRIPPO_SOURCE; export SCRIPPO_SANDBOX_ROOT=$scratch/source; compile "$file") > "$output"
      bash -n "$output"
      chmod 755 "$output"
    fi
  done
  if [[ $mode == scripts ]]; then
    cp -R "$scratch/source/." "$scratch/output/"
  fi
  jq -n --arg repository "$repo" --arg tag "$tag" --arg commit "$ref" --arg kind "$mode" \
    '{repository: $repository, tag: $tag, commit: $commit, kind: $kind}' > "$scratch/output/.scrippo-install.json"
  # Claim the destination atomically so simultaneous installers cannot merge.
  mkdir -- "$target" || { package_error "destination already exists: $target"; exit 1; }
  claimed=true
  mv -- "$scratch/output" "$target/content"
  printf 'Installed %s %s (%s) to %s/content\n' "$name" "$tag" "$mode" "$target"
)

function package_validate_info {
  local file=$1 version=$2 tree node
  local -a nodes=() args=()
  tree=$(set +eu; ast "$file") || return 1
  mapfile -t nodes < <(jq -c '.body[] | select(.type == "macro" and .name == "info")' <<< "$tree")
  (( ${#nodes[@]} == 1 )) || { package_error "$file: expected exactly one top-level @info"; return 1; }
  node=${nodes[0]}
  mapfile -t args < <(jq -r '.args[]' <<< "$node")
  macro_info "$node" "${args[@]}"
  [[ $_scrippo_info_name == "$(basename "$file" .scrippo)" ]] || { package_error "$file: @info name must match the filename"; return 1; }
  [[ $_scrippo_info_version == "$version" ]] || { package_error "$file: @info version must match tag $version"; return 1; }
}
