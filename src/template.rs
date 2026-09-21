//! Scaffold contents and file writers for `carry new` / `carry init`.
//!
//! Templates are `&'static str` constants; `{name}`, `{type}` and `{ident}` are
//! substituted with plain `str::replace` (no `format!` on user input).
//!
//! `{ident}` is the C++ identifier form of the project name: the name rule
//! allows `-`, which is not a valid C++ identifier character, so dashes become
//! underscores in namespaces/identifiers (file paths keep the raw name).

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::manifest::{self, Build, Manifest, Package};

/// CMakeUserPresets.json: conan's CMakeToolchain writes it next to the
/// conanfile/CMakeLists (project root) on every `conan install` (I001).
/// compile_commands.json: carry's editor-facing compile database, copied to
/// the project root on every build (D13).
pub const GITIGNORE: &str = "build/\nCMakeUserPresets.json\ncompile_commands.json\n";

/// main.cpp for type "bin" (no library to call).
pub const MAIN_CPP_BIN: &str = "\
#include <iostream>

int main() {
  std::cout << \"Hello from {name}!\" << std::endl;
  return 0;
}
";

/// main.cpp for type "both" (calls the library's greeting).
pub const MAIN_CPP_BOTH: &str = "\
#include <iostream>

#include \"{name}/{name}.hpp\"

int main() {
  std::cout << {ident}::greeting() << std::endl;
  return 0;
}
";

/// include/<name>/<name>.hpp
pub const LIB_HPP: &str = "\
#pragma once

#include <string>

namespace {ident} {

std::string greeting();

}  // namespace {ident}
";

/// src/lib.cpp
pub const LIB_CPP: &str = "\
#include \"{name}/{name}.hpp\"

namespace {ident} {

std::string greeting() {
  return \"Hello from {name}!\";
}

}  // namespace {ident}
";

/// tests/test_main.cpp — trivial assert-based test binary, no framework.
pub const TEST_MAIN_CPP: &str = "\
#include <cassert>
#include <cstdio>

int main() {
  assert(1 + 1 == 2);  // placeholder — real project tests go here
  std::puts(\"all tests passed\");
  return 0;
}
";

fn render(template: &str, name: &str, pkg_type: &str, ident: &str) -> String {
    template
        .replace("{name}", name)
        .replace("{type}", pkg_type)
        .replace("{ident}", ident)
}

/// C++ identifier form of the project name (dashes → underscores).
pub fn cpp_ident(name: &str) -> String {
    name.replace('-', "_")
}

/// All scaffold files except CARRY.toml (written separately via `Manifest::save`):
/// (relative path, rendered content), in creation order. `conanfile.txt` and
/// `CMakeLists.txt` are real renders from the manifest (T003), not stubs.
fn files_for(m: &Manifest) -> Vec<(String, String)> {
    let name = &m.package.name;
    let pkg_type = &m.package.kind;
    let ident = cpp_ident(name);
    let mut files: Vec<(String, String)> = vec![
        (".gitignore".to_string(), GITIGNORE.to_string()),
        ("conanfile.txt".to_string(), crate::generate::render_conanfile(m)),
        ("CMakeLists.txt".to_string(), crate::generate::render_cmake(m)),
    ];
    let main = if pkg_type == "both" { MAIN_CPP_BOTH } else { MAIN_CPP_BIN };
    files.push((
        "src/main.cpp".to_string(),
        render(main, name, pkg_type, &ident),
    ));
    if pkg_type == "lib" || pkg_type == "both" {
        files.push((
            format!("include/{name}/{name}.hpp"),
            render(LIB_HPP, name, pkg_type, &ident),
        ));
        files.push((
            "src/lib.cpp".to_string(),
            render(LIB_CPP, name, pkg_type, &ident),
        ));
    }
    files.push(("tests/test_main.cpp".to_string(), TEST_MAIN_CPP.to_string()));
    files
}

/// Create the scaffold under `root`. Returns the created paths, relative to
/// `root`, in creation order.
fn write_scaffold(root: &Path, name: &str, pkg_type: &str) -> Result<Vec<String>> {
    let manifest = Manifest {
        package: Package {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            kind: pkg_type.to_string(),
        },
        build: Build::default(),
        dependencies: std::collections::BTreeMap::new(),
    };
    let mut created = Vec::new();
    let carry_toml = root.join("CARRY.toml");
    manifest.save(&carry_toml).with_context(|| {
        format!(
            "writing {}",
            carry_toml.strip_prefix(root).unwrap_or(&carry_toml).display()
        )
    })?;
    created.push("CARRY.toml".to_string());
    for (rel, content) in files_for(&manifest) {
        let path = root.join(&rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating {}", parent.display()))?;
        }
        std::fs::write(&path, content)
            .with_context(|| format!("writing {}", path.display()))?;
        created.push(rel);
    }
    Ok(created)
}

fn print_created(created: &[String]) {
    for rel in created {
        println!("  {rel}");
    }
}

/// `carry new <name> [--lib]` — create `./<name>/` with the full scaffold.
pub fn cmd_new(name: &str, lib: bool) -> Result<()> {
    manifest::validate_name(name)?;
    let pkg_type = if lib { "both" } else { "bin" };
    let root = Path::new(name);
    match std::fs::metadata(root) {
        Ok(meta) => {
            if !meta.is_dir() {
                bail!("{name} exists and is not a directory");
            }
            if root.join("CARRY.toml").is_file() {
                bail!("{name} is already a carry project\nhelp: cd into it and run 'carry build'");
            }
            let has_entries = std::fs::read_dir(root)
                .with_context(|| format!("reading {name}"))?
                .any(|entry| entry.is_ok());
            if has_entries {
                bail!("{name} dir exists and is not a carry project\nhelp: remove it or pick a different name");
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
        Err(err) => return Err(err).with_context(|| format!("checking {name}")),
    }
    let created = write_scaffold(root, name, pkg_type)
        .with_context(|| format!("creating project in {name}"))?;
    println!("created carry project {name} (type: {pkg_type})");
    print_created(&created);
    println!();
    println!("next: cd {name} && carry build");
    Ok(())
}

/// `carry init` — turn the current directory into a carry project (type "bin",
/// name from the directory name).
pub fn cmd_init() -> Result<()> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    let name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    manifest::validate_name(&name)?;
    if cwd.join("CARRY.toml").is_file() {
        bail!("already a carry project\nhelp: run 'carry build' instead");
    }
    let created = write_scaffold(&cwd, &name, "bin")
        .with_context(|| format!("initializing carry project in {}", cwd.display()))?;
    println!("initialized carry project {name} (type: bin)");
    print_created(&created);
    println!();
    println!("next: carry build");
    Ok(())
}
