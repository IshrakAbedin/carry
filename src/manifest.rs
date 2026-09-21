//! CARRY.toml manifest: serde schema, load/save, validation, conan ref mapping,
//! and the format-preserving `add`/`remove` dependency commands (T006).
//!
//! The manifest is the single source of truth for a carry project (DESIGN.md §4).
//! `save` goes through `toml_edit` so user comments and formatting survive edits.

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

    /// Add or update a dependency, saving `path` via a `toml_edit` round-trip
    /// so every other line (comments, spacing) survives. Same version → no
    /// write, "already up to date". Prints what changed and, after a write,
    /// the `run: carry build` hint (the generated files pick the change up on
    /// the next build — this never builds itself).
    pub fn add_dep(&mut self, path: &Path, pkg: &str, version: &str) -> Result<()> {
        validate_labeled_name("dependency", pkg)?;
        match self.dependencies.get(pkg) {
            Some(existing) if existing == version => {
                println!("{pkg} already up to date");
                return Ok(());
            }
            Some(old) => {
                let old = old.clone();
                self.dependencies.insert(pkg.to_string(), version.to_string());
                self.save(path)?;
                println!("updated {pkg} {} → {}", display_version(&old), display_version(version));
            }
            None => {
                self.dependencies.insert(pkg.to_string(), version.to_string());
                self.save(path)?;
                if version.is_empty() {
                    println!("added {pkg}");
                } else {
                    println!("added {pkg} {version}");
                }
            }
        }
        println!("run: carry build");
        Ok(())
    }

    /// Remove a dependency, saving `path` via a `toml_edit` round-trip. The
    /// `[dependencies]` table is left in place even when it becomes empty
    /// (stable diff; the user may re-add). A `pkg@version` spec is tolerated —
    /// only the name is used. Errors if the dependency is not present.
    pub fn remove_dep(&mut self, path: &Path, pkg: &str) -> Result<()> {
        let name = pkg.rsplit_once('@').map_or(pkg, |(n, _)| n);
        let Some(old) = self.dependencies.remove(name) else {
            anyhow::bail!("dependency '{name}' not found in CARRY.toml");
        };
        self.save(path)?;
        println!("removed {name} {}", display_version(&old));
        println!("run: carry build");
        Ok(())
    }
}

/// Parse a dependency spec `pkg` or `pkg@version`, splitting on the **last**
/// `@`, and validate the name with the manifest name rule. The version part
/// may be plain (`10.2.1`) or a range (`>=10`); bare `pkg` stores an empty
/// version (maps to the "any" range via `conan_ref`). An `@` with an empty
/// version after it is a typo and is rejected (T007).
pub fn parse_dep_spec(spec: &str) -> Result<(String, String)> {
    let (name, version) = match spec.rsplit_once('@') {
        Some((_, "")) => anyhow::bail!(
            "missing version after '@' in \"{spec}\"\n\
             help: use pkg@version, e.g. fmt@10.2.1 — or drop the '@' for any version"
        ),
        Some((n, v)) => (n, v.to_string()),
        None => (spec, String::new()),
    };
    validate_labeled_name("dependency", name)?;
    Ok((name.to_string(), version))
}

/// Empty version (bare `pkg`) is shown as `(any)` in change messages.
fn display_version(version: &str) -> &str {
    if version.is_empty() {
        "(any)"
    } else {
        version
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

/// Validate a name against the `[a-z0-9_][a-z0-9_-]*` rule (DESIGN.md §4),
/// with a `help:` hint on failure. `label` is "project" or "dependency".
fn validate_labeled_name(label: &str, name: &str) -> Result<()> {
    anyhow::ensure!(
        is_valid_name(name),
        "invalid {label} name {name:?}\n\
         help: names must match [a-z0-9_][a-z0-9_-] \
         (lowercase letters, digits, '_', '-'; must not start with '-')"
    );
    Ok(())
}

/// Validate a project name, with a `help:` hint on failure.
pub fn validate_name(name: &str) -> Result<()> {
    validate_labeled_name("project", name)
}

/// Map a manifest dependency value to a Conan reference (DESIGN.md §4):
/// plain version → `pkg/ver`, range (`>` `<` `=` `^` `~` prefix) → `pkg/[range]`,
/// empty → `pkg/[>=0]`. A bare `pkg/*` is *not* a valid require in Conan 2
/// (2.32.0 rejects it as unresolvable even when matching versions are
/// cached), so "any version" uses the `[>=0]` range — the highest available
/// version, cache preferred (D11, T007).
pub fn conan_ref(pkg: &str, version: &str) -> String {
    match version.chars().next() {
        Some('>' | '<' | '=' | '^' | '~') => format!("{pkg}/[{version}]"),
        None => format!("{pkg}/[>=0]"),
        _ => format!("{pkg}/{version}"),
    }
}

/// Map the manifest's `cpp_std` to a Conan `compiler.cppstd` setting (D12):
/// the dependency graph must be resolved with the standard the generated
/// CMakeLists compiles with. The pinned GCC toolchain (D6) uses the GNU
/// dialect variants, so `c++23` → `gnu23`; a value that already carries the
/// `gnu` prefix passes through unchanged.
pub fn conan_cppstd(standard: &str) -> String {
    let std = standard.strip_prefix("c++").unwrap_or(standard);
    if std.starts_with("gnu") {
        std.to_string()
    } else {
        format!("gnu{std}")
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

    /// A manifest with comments and odd spacing, and NO `[dependencies]`
    /// table (the harder add case: the table must be created).
    const COMMENTED_NO_DEPS: &str = "\
# my project
[package]
name = \"demo\"
# pinned version
version = \"0.1.0\"
type = \"bin\"

[build]
cpp_std = \"c++23\"
c_std   = \"c11\"
";

    fn write_manifest(dir_name: &str, content: &str) -> std::path::PathBuf {
        let dir = test_dir(dir_name);
        let path = dir.join("CARRY.toml");
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn parse_dep_spec_forms() {
        assert_eq!(parse_dep_spec("fmt").unwrap(), ("fmt".into(), String::new()));
        assert_eq!(
            parse_dep_spec("fmt@10.2.1").unwrap(),
            ("fmt".into(), "10.2.1".into())
        );
        assert_eq!(
            parse_dep_spec("fmt@>=10").unwrap(),
            ("fmt".into(), ">=10".into())
        );
        // Split on the LAST '@': a '@' in the name part → invalid name.
        assert!(parse_dep_spec("a@b@c").is_err());
        // A trailing '@' is a typo, not "any version" (T007).
        let err = parse_dep_spec("fmt@").unwrap_err().to_string();
        assert!(err.contains("missing version after '@'"), "{err}");
        assert!(err.contains("help:"), "{err}");
        for bad in ["Bad-Pkg", "-x", "", "bad name", "bad/name", "böhm"] {
            let err = parse_dep_spec(bad).unwrap_err().to_string();
            assert!(
                err.contains("invalid dependency name") && err.contains("help:"),
                "{bad:?}: {err}"
            );
        }
    }

    #[test]
    fn add_dep_new_appends_table() {
        let path = write_manifest("manifest-add-new", COMMENTED_NO_DEPS);
        let mut m = Manifest::load(&path).unwrap();
        m.add_dep(&path, "fmt", "10.2.1").unwrap();
        assert_eq!(m.dependencies.get("fmt").map(String::as_str), Some("10.2.1"));
        let text = std::fs::read_to_string(&path).unwrap();
        // New table appended at the end with a blank line before it.
        assert!(text.ends_with("[dependencies]\nfmt = \"10.2.1\"\n"), "\n{text}");
    }

    #[test]
    fn add_dep_replaces_version_in_place() {
        let path = write_manifest("manifest-add-replace", FULL);
        let mut m = Manifest::load(&path).unwrap();
        m.add_dep(&path, "fmt", "11.0.0").unwrap();
        assert_eq!(m.dependencies.get("fmt").map(String::as_str), Some("11.0.0"));
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("fmt = \"11.0.0\""), "\n{text}");
        assert!(!text.contains("10.2.1"), "old version must be gone:\n{text}");
        // The other dependency is untouched.
        assert!(text.contains("zeta = \">=1.0\""), "\n{text}");
    }

    #[test]
    fn add_dep_same_version_writes_nothing() {
        let path = write_manifest("manifest-add-same", FULL);
        let before = std::fs::read_to_string(&path).unwrap();
        let mtime_before = std::fs::metadata(&path).unwrap().modified().unwrap();
        let mut m = Manifest::load(&path).unwrap();
        m.add_dep(&path, "fmt", "10.2.1").unwrap();
        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            before,
            "content must be unchanged"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().modified().unwrap(),
            mtime_before,
            "file must not be rewritten at all"
        );
    }

    #[test]
    fn add_dep_rejects_invalid_name() {
        let path = write_manifest("manifest-add-badname", FULL);
        let mut m = Manifest::load(&path).unwrap();
        let err = m.add_dep(&path, "Bad-Pkg", "1.0").unwrap_err().to_string();
        assert!(
            err.contains("invalid dependency name") && err.contains("help:"),
            "{err}"
        );
        // The manifest is untouched.
        assert_eq!(std::fs::read_to_string(&path).unwrap(), FULL);
    }

    #[test]
    fn remove_dep_present_keeps_table_header() {
        let path = write_manifest("manifest-remove-present", FULL);
        let mut m = Manifest::load(&path).unwrap();
        m.remove_dep(&path, "fmt").unwrap();
        assert!(!m.dependencies.contains_key("fmt"));
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("[dependencies]"), "table header must remain:\n{text}");
        assert!(!text.contains("fmt"), "key must be gone:\n{text}");
        assert!(text.contains("zeta = \">=1.0\""), "other deps untouched:\n{text}");
        // A pkg@version spec is tolerated — only the name matters.
        m.remove_dep(&path, "zeta@>=1.0").unwrap();
        assert!(m.dependencies.is_empty());
    }

    #[test]
    fn remove_dep_absent_errors_and_writes_nothing() {
        let path = write_manifest("manifest-remove-absent", FULL);
        let mut m = Manifest::load(&path).unwrap();
        let err = m.remove_dep(&path, "boost").unwrap_err();
        assert_eq!(err.to_string(), "dependency 'boost' not found in CARRY.toml");
        assert_eq!(std::fs::read_to_string(&path).unwrap(), FULL, "file must be unchanged");
    }

    /// A CARRY.toml with comments and odd spacing keeps comments/spacing after
    /// add + remove (asserted on the exact raw string).
    #[test]
    fn add_remove_preserves_comments_and_spacing() {
        let path = write_manifest("manifest-format-preserve", COMMENTED_NO_DEPS);
        let mut m = Manifest::load(&path).unwrap();

        m.add_dep(&path, "fmt", "10.2.1").unwrap();
        let after_add = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            after_add,
            "# my project\n\
             [package]\n\
             name = \"demo\"\n\
             # pinned version\n\
             version = \"0.1.0\"\n\
             type = \"bin\"\n\
             \n\
             [build]\n\
             cpp_std = \"c++23\"\n\
             c_std   = \"c11\"\n\
             \n\
             [dependencies]\n\
             fmt = \"10.2.1\"\n"
        );

        m.remove_dep(&path, "fmt").unwrap();
        let after_remove = std::fs::read_to_string(&path).unwrap();
        // Key gone; the (now empty) table header remains.
        assert_eq!(
            after_remove,
            "# my project\n\
             [package]\n\
             name = \"demo\"\n\
             # pinned version\n\
             version = \"0.1.0\"\n\
             type = \"bin\"\n\
             \n\
             [build]\n\
             cpp_std = \"c++23\"\n\
             c_std   = \"c11\"\n\
             \n\
             [dependencies]\n"
        );
    }

    #[test]
    fn conan_cppstd_maps_to_gnu_variant() {
        assert_eq!(conan_cppstd("c++23"), "gnu23");
        assert_eq!(conan_cppstd("c++20"), "gnu20");
        assert_eq!(conan_cppstd("c++17"), "gnu17");
        // Already in Conan's gnu form: passes through.
        assert_eq!(conan_cppstd("gnu23"), "gnu23");
        // Bare number (the CMake mapping accepts it too).
        assert_eq!(conan_cppstd("23"), "gnu23");
    }

    #[test]
    fn conan_ref_branches() {
        assert_eq!(conan_ref("fmt", "10.2.1"), "fmt/10.2.1");
        assert_eq!(conan_ref("fmt", ">=10"), "fmt/[>=10]");
        assert_eq!(conan_ref("z", "<=2"), "z/[<=2]");
        assert_eq!(conan_ref("z", "=1.2.3"), "z/[=1.2.3]");
        assert_eq!(conan_ref("z", "^1"), "z/[^1]");
        assert_eq!(conan_ref("z", "~1.2"), "z/[~1.2]");
        // Bare pkg → "any" range (a bare `fmt/*` is not a valid require, D11).
        assert_eq!(conan_ref("fmt", ""), "fmt/[>=0]");
    }
}
