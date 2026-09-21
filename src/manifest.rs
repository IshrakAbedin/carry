//! CARRY.toml manifest: serde schema, load/save, validation, conan ref mapping.
//!
//! The manifest is the single source of truth for a carry project (DESIGN.md §4).
//! `save` goes through `toml_edit` so user comments and formatting survive edits
//! (the format-preserving `add`/`remove` commands build on this in T006).

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use toml_edit::Item;

const VERSION_DEFAULT: &str = "0.1.0";
const TYPE_DEFAULT: &str = "bin";
const CPP_STD_DEFAULT: &str = "c++23";
const C_STD_DEFAULT: &str = "c11";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub package: Package,
    #[serde(default)]
    pub build: Build,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(rename = "type", default = "default_type")]
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Build {
    #[serde(default = "default_cpp_std")]
    pub cpp_std: String,
    #[serde(default = "default_c_std")]
    pub c_std: String,
}

fn default_version() -> String {
    VERSION_DEFAULT.to_string()
}

fn default_type() -> String {
    TYPE_DEFAULT.to_string()
}

fn default_cpp_std() -> String {
    CPP_STD_DEFAULT.to_string()
}

fn default_c_std() -> String {
    C_STD_DEFAULT.to_string()
}

impl Default for Package {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: default_version(),
            kind: default_type(),
        }
    }
}

impl Default for Build {
    fn default() -> Self {
        Self {
            cpp_std: default_cpp_std(),
            c_std: default_c_std(),
        }
    }
}

impl Manifest {
    /// Read and parse `CARRY.toml` from `path`, then validate it.
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("reading {}", path.display()))?;
        let manifest: Self = toml::from_str(&text)
            .with_context(|| format!("parsing {}", path.display()))?;
        manifest.validate()?;
        Ok(manifest)
    }

    /// Write the manifest to `path` via `toml_edit`: an existing file is parsed
    /// first and updated in place (comments/formatting survive); a missing or
    /// empty file is written from scratch.
    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let existing = std::fs::read_to_string(path).ok();
        let mut doc: toml_edit::DocumentMut = match existing.as_deref() {
            Some(text) if !text.trim().is_empty() => text
                .parse()
                .with_context(|| format!("parsing existing {}", path.display()))?,
            _ => toml::to_string_pretty(self)
                .context("serializing manifest")?
                .parse()
                .context("serializing manifest")?,
        };
        apply_to_doc(self, &mut doc);
        if let Some(parent) = path.parent() && !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(path, doc.to_string())
            .with_context(|| format!("writing {}", path.display()))?;
        Ok(())
    }

    /// Check schema rules; returns errors with `help:` hints (DESIGN.md §8).
    pub fn validate(&self) -> Result<()> {
        validate_name(&self.package.name)?;
        if self.package.version.is_empty() {
            anyhow::bail!(
                "package.version is empty\n\
                 help: set a semver string, e.g. version = \"0.1.0\""
            );
        }
        match self.package.kind.as_str() {
            "bin" | "lib" | "both" => {}
            other => anyhow::bail!(
                "invalid package type {other:?}\n\
                 help: type must be one of: \"bin\", \"lib\", \"both\""
            ),
        }
        Ok(())
    }
}

/// Set every manifest value into an (existing or fresh) toml_edit document.
fn apply_to_doc(m: &Manifest, doc: &mut toml_edit::DocumentMut) {
    let pkg = ensure_table(doc, "package");
    set_str(pkg, "name", &m.package.name);
    set_str(pkg, "version", &m.package.version);
    set_str(pkg, "type", &m.package.kind);
    let build = ensure_table(doc, "build");
    set_str(build, "cpp_std", &m.build.cpp_std);
    set_str(build, "c_std", &m.build.c_std);
    let deps = ensure_table(doc, "dependencies");
    for (name, version) in &m.dependencies {
        set_str(deps, name, version);
    }
    let stale: Vec<String> = deps
        .iter()
        .map(|(name, _)| name.to_string())
        .filter(|name| !m.dependencies.contains_key(name))
        .collect();
    for name in stale {
        deps.remove(&name);
    }
}

/// Set a string key in a table. When the key already holds a string, only the
/// value is replaced — the key (which owns the line's leading comments in
/// toml_edit's model) and the value's trailing decoration are preserved.
/// Other cases insert a fresh key.
fn set_str(table: &mut toml_edit::Table, key: &str, value: &str) {
    match table.get_mut(key) {
        Some(Item::Value(v)) if v.is_str() => {
            let old_decor = v.decor().clone();
            let mut new_value =
                toml_edit::Value::String(toml_edit::Formatted::new(value.to_string()));
            let decor = new_value.decor_mut();
            if let Some(prefix) = old_decor.prefix() {
                decor.set_prefix(prefix.clone());
            }
            if let Some(suffix) = old_decor.suffix() {
                decor.set_suffix(suffix.clone());
            }
            *v = new_value;
        }
        _ => {
            table.insert(key, toml_edit::value(value.to_string()));
        }
    }
}

/// Return the named root table of `doc`, creating it if missing or non-table.
fn ensure_table<'a>(doc: &'a mut toml_edit::DocumentMut, key: &str) -> &'a mut toml_edit::Table {
    let is_table = doc.get(key).and_then(Item::as_table).is_some();
    if !is_table {
        doc.insert(key, Item::Table(toml_edit::Table::new()));
    }
    doc.get_mut(key)
        .and_then(Item::as_table_mut)
        .expect("table (re)created above")
}

/// A valid project name: `[a-z0-9_][a-z0-9_-]*` (DESIGN.md §4).
pub fn is_valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    let first = matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    first && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Validate a project name, with a `help:` hint on failure.
pub fn validate_name(name: &str) -> Result<()> {
    anyhow::ensure!(
        is_valid_name(name),
        "invalid project name {name:?}\n\
         help: names must match [a-z0-9_][a-z0-9_-] \
         (lowercase letters, digits, '_', '-'; must not start with '-')"
    );
    Ok(())
}

/// Map a manifest dependency value to a Conan reference (DESIGN.md §4):
/// plain version → `pkg/ver`, range (`>` `<` `=` `^` `~` prefix) → `pkg/[range]`,
/// empty → `pkg/*`.
pub fn conan_ref(pkg: &str, version: &str) -> String {
    match version.chars().next() {
        Some('>' | '<' | '=' | '^' | '~') => format!("{pkg}/[{version}]"),
        None => format!("{pkg}/*"),
        _ => format!("{pkg}/{version}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL: &str = "\
[package]
name = \"demo\"
version = \"0.1.0\"
type = \"both\"

[build]
cpp_std = \"c++23\"
c_std = \"c11\"

[dependencies]
fmt = \"10.2.1\"
zeta = \">=1.0\"
";

    /// Deterministic scratch dir under target/ (gitignored) for file-based tests.
    fn test_dir(name: &str) -> std::path::PathBuf {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("tmp")
            .join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn parses_full_manifest() {
        let m: Manifest = toml::from_str(FULL).unwrap();
        assert_eq!(m.package.name, "demo");
        assert_eq!(m.package.version, "0.1.0");
        assert_eq!(m.package.kind, "both");
        assert_eq!(m.build.cpp_std, "c++23");
        assert_eq!(m.build.c_std, "c11");
        assert_eq!(m.dependencies.get("fmt").map(String::as_str), Some("10.2.1"));
        assert_eq!(m.dependencies.get("zeta").map(String::as_str), Some(">=1.0"));
        assert_eq!(m.dependencies.len(), 2);
        m.validate().unwrap();
    }

    #[test]
    fn parses_minimal_manifest_with_defaults() {
        let m: Manifest = toml::from_str("[package]\nname = \"demo\"\n").unwrap();
        assert_eq!(m.package.version, "0.1.0");
        assert_eq!(m.package.kind, "bin");
        assert_eq!(m.build.cpp_std, "c++23");
        assert_eq!(m.build.c_std, "c11");
        assert!(m.dependencies.is_empty());
        m.validate().unwrap();
    }

    /// Exact shape `new`/`init` scaffold (incl. empty `[dependencies]` table).
    #[test]
    fn parses_scaffold_shape_manifest() {
        let m: Manifest = toml::from_str(
            "[package]\n\
             name = \"demo\"\n\
             version = \"0.1.0\"\n\
             type = \"both\"\n\
             \n\
             [build]\n\
             cpp_std = \"c++23\"\n\
             c_std = \"c11\"\n\
             \n\
             [dependencies]\n",
        )
        .unwrap();
        assert!(m.dependencies.is_empty());
        m.validate().unwrap();
    }

    #[test]
    fn save_load_roundtrip() {
        let dir = test_dir("manifest-roundtrip");
        let path = dir.join("CARRY.toml");
        let m: Manifest = toml::from_str(FULL).unwrap();
        m.save(&path).unwrap();
        let m2 = Manifest::load(&path).unwrap();
        assert_eq!(m, m2);
    }

    #[test]
    fn save_preserves_existing_comments() {
        let dir = test_dir("manifest-comments");
        let path = dir.join("CARRY.toml");
        std::fs::write(
            &path,
            "# my project\n\
             [package]\n\
             name = \"demo\"\n\
             # pinned version\n\
             version = \"0.1.0\"\n\
             type = \"bin\"\n",
        )
        .unwrap();
        let mut m = Manifest::load(&path).unwrap();
        m.dependencies.insert("fmt".to_string(), "10.2.1".to_string());
        m.save(&path).unwrap();
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("# my project"), "header comment lost:\n{text}");
        assert!(text.contains("# pinned version"), "inline comment lost:\n{text}");
        let m2 = Manifest::load(&path).unwrap();
        assert_eq!(m2.dependencies.get("fmt").map(String::as_str), Some("10.2.1"));
    }

    #[test]
    fn save_removes_stale_dependencies() {
        let dir = test_dir("manifest-stale-deps");
        let path = dir.join("CARRY.toml");
        std::fs::write(
            &path,
            "[package]\nname = \"demo\"\n\n[dependencies]\nold = \"1.0\"\n",
        )
        .unwrap();
        let mut m = Manifest::load(&path).unwrap();
        m.dependencies.clear();
        m.dependencies.insert("fmt".to_string(), "10.2.1".to_string());
        m.save(&path).unwrap();
        let m2 = Manifest::load(&path).unwrap();
        assert!(!m2.dependencies.contains_key("old"));
        assert_eq!(m2.dependencies.get("fmt").map(String::as_str), Some("10.2.1"));
    }

    #[test]
    fn name_validation_rules() {
        for ok in ["demo", "a", "foo-bar", "_x", "9lives", "a_b-c"] {
            assert!(is_valid_name(ok), "{ok:?} should be valid");
        }
        for bad in ["", "-leading", "Bad", "bad name", "bad/name", "böhm", "a?"] {
            assert!(!is_valid_name(bad), "{bad:?} should be invalid");
        }
    }

    #[test]
    fn validate_rejects_bad_name_type_version() {
        for text in [
            "[package]\nname = \"\"\n",
            "[package]\nname = \"Bad-Name\"\n",
            "[package]\nname = \"demo\"\nversion = \"\"\n",
            "[package]\nname = \"demo\"\ntype = \"exe\"\n",
        ] {
            let m: Manifest = toml::from_str(text).unwrap();
            let err = m.validate().unwrap_err().to_string();
            assert!(err.contains("help:"), "missing help hint in: {err}");
        }
    }

    #[test]
    fn conan_ref_branches() {
        assert_eq!(conan_ref("fmt", "10.2.1"), "fmt/10.2.1");
        assert_eq!(conan_ref("fmt", ">=10"), "fmt/[>=10]");
        assert_eq!(conan_ref("z", "<=2"), "z/[<=2]");
        assert_eq!(conan_ref("z", "=1.2.3"), "z/[=1.2.3]");
        assert_eq!(conan_ref("z", "^1"), "z/[^1]");
        assert_eq!(conan_ref("z", "~1.2"), "z/[~1.2]");
        assert_eq!(conan_ref("fmt", ""), "fmt/*");
    }
}
