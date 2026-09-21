//! E2E tests: spawn the real `carry` binary in `tempfile` dirs.
//!
//! The default suite stays hermetic (D8): only tests that need the C++
//! toolchain and/or Conan Center network are marked `#[ignore]`. Run them
//! with `cargo test -- --ignored`. `CARRY_BIN` overrides the binary under
//! test (defaults to the cargo-built one).

use std::path::Path;
use std::process::Command;

fn carry_bin() -> String {
    std::env::var("CARRY_BIN").unwrap_or_else(|_| env!("CARGO_BIN_EXE_carry").to_string())
}

/// Spawn `carry <args>` with cwd `root`; capture stdout/stderr.
fn carry(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(carry_bin())
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|err| panic!("spawning carry: {err}"))
}

/// Spawn and require exit 0; returns (stdout, stderr) as lossy strings.
fn carry_ok(root: &Path, args: &[&str]) -> (String, String) {
    let out = carry(root, args);
    let (stdout, stderr) = (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    );
    assert!(
        out.status.success(),
        "carry {args:?} failed (status {})\nstdout:\n{stdout}\nstderr:\n{stderr}",
        out.status
    );
    (stdout, stderr)
}

/// `carry new --lib` scaffold (DESIGN.md §3) — hermetic: no tools, no network.
#[test]
fn new_scaffold_layout() {
    let dir = tempfile::tempdir().unwrap();
    let (stdout, _) = carry_ok(dir.path(), &["new", "fmtapp", "--lib"]);
    assert!(stdout.contains("created carry project fmtapp (type: both)"), "{stdout}");

    let root = dir.path().join("fmtapp");
    for rel in [
        "CARRY.toml",
        "conanfile.txt",
        "CMakeLists.txt",
        ".gitignore",
        "src/main.cpp",
        "src/lib.cpp",
        "include/fmtapp/fmtapp.hpp",
        "tests/test_main.cpp",
    ] {
        assert!(root.join(rel).is_file(), "missing {rel}");
    }

    let manifest = std::fs::read_to_string(root.join("CARRY.toml")).unwrap();
    assert!(manifest.contains("name = \"fmtapp\""), "{manifest}");
    assert!(manifest.contains("type = \"both\""), "{manifest}");

    // I001: conan drops CMakeUserPresets.json in the project root on every
    // build — the scaffold .gitignore must cover it (and the build dir).
    let gitignore = std::fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(gitignore.contains("build/"), "{gitignore}");
    assert!(gitignore.contains("CMakeUserPresets.json"), "{gitignore}");
}

/// `carry add`/`remove` through the real binary — hermetic: these edit
/// CARRY.toml only and never build.
#[test]
fn add_remove_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    carry_ok(dir.path(), &["new", "demo"]);
    let root = dir.path().join("demo");

    let (stdout, _) = carry_ok(&root, &["add", "fmt@10.2.1"]);
    assert!(stdout.contains("added fmt 10.2.1"), "{stdout}");
    let manifest = std::fs::read_to_string(root.join("CARRY.toml")).unwrap();
    assert!(manifest.contains("fmt = \"10.2.1\""), "{manifest}");

    let (stdout, _) = carry_ok(&root, &["add", "fmt@10.2.1"]);
    assert!(stdout.contains("fmt already up to date"), "{stdout}");

    let (stdout, _) = carry_ok(&root, &["add", "fmt@11.0.0"]);
    assert!(stdout.contains("updated fmt 10.2.1 → 11.0.0"), "{stdout}");

    let (stdout, _) = carry_ok(&root, &["remove", "fmt"]);
    assert!(stdout.contains("removed fmt 11.0.0"), "{stdout}");
    let manifest = std::fs::read_to_string(root.join("CARRY.toml")).unwrap();
    assert!(!manifest.contains("fmt ="), "{manifest}");

    // Absent dependency → carry error (exit 1), clean message.
    let out = carry(&root, &["remove", "fmt"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("dependency 'fmt' not found in CARRY.toml"), "{stderr}");

    // A trailing '@' promises a version: rejected, not silently "any" (D11).
    let out = carry(&root, &["add", "fmt@"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("missing version after '@'"), "{stderr}");
    let manifest = std::fs::read_to_string(root.join("CARRY.toml")).unwrap();
    assert!(!manifest.contains("fmt ="), "{manifest}");
}

const FMT_MAIN_CPP: &str = "\
#include <iostream>

#include <fmt/format.h>

#include \"fmtapp/fmtapp.hpp\"

int main() {
  std::cout << fmtapp::greeting() << std::endl;
  fmt::print(\"answer: {}\\n\", 42);
  return 0;
}
";

const FMT_LIB_CPP: &str = "\
#include \"fmtapp/fmtapp.hpp\"

#include <fmt/format.h>

namespace fmtapp {

std::string greeting() {
  return fmt::format(\"Hello from fmtapp!\");
}

}  // namespace fmtapp
";

const FMT_TEST_CPP: &str = "\
#include <cassert>
#include <cstdio>

#include <fmt/format.h>

int main() {
  assert(fmt::format(\"{}+{}\", 1, 1) == \"1+1\");  // exercises the fmt::fmt link
  std::puts(\"all tests passed\");
  return 0;
}
";

const ORIG_MAIN_CPP: &str = "\
#include <iostream>

#include \"fmtapp/fmtapp.hpp\"

int main() {
  std::cout << fmtapp::greeting() << std::endl;
  return 0;
}
";

const ORIG_LIB_CPP: &str = "\
#include \"fmtapp/fmtapp.hpp\"

namespace fmtapp {

std::string greeting() {
  return \"Hello from fmtapp!\";
}

}  // namespace fmtapp
";

const ORIG_TEST_CPP: &str = "\
#include <cassert>
#include <cstdio>

int main() {
  assert(1 + 1 == 2);  // placeholder — real project tests go here
  std::puts(\"all tests passed\");
  return 0;
}
";

const GLAZE_MAIN_CPP: &str = "\
#include \"glaze/glaze.hpp\"

#include <iostream>
#include <string>

struct person {
  std::string name;
  int age;
};

int main() {
  person p{\"Alice\", 42};
  std::string json;
  if (auto ec = glz::write_json(p, json)) {
    std::cerr << glz::format_error(ec, json) << std::endl;
    return 1;
  }
  std::cout << json << std::endl;
  return 0;
}
";

/// The full loop against real Conan Center (T007 step 2): new → add fmt →
/// build Debug → run → build Release → test → remove fmt → revert → build.
/// Needs the C++ toolchain and (first time only) Conan Center network, so it
/// is `#[ignore]`d per D8.
#[test]
#[ignore = "needs C++ toolchain + Conan Center network (D8)"]
fn fmt_full_loop() {
    let dir = tempfile::tempdir().unwrap();
    carry_ok(dir.path(), &["new", "fmtapp", "--lib"]);
    let root = dir.path().join("fmtapp");

    let (stdout, _) = carry_ok(&root, &["add", "fmt@10.2.1"]);
    assert!(stdout.contains("added fmt 10.2.1"), "{stdout}");

    // Use fmt in the lib (D10: the lib target must see it), the bin, and the
    // test binary.
    for (rel, content) in [
        ("src/main.cpp", FMT_MAIN_CPP),
        ("src/lib.cpp", FMT_LIB_CPP),
        ("tests/test_main.cpp", FMT_TEST_CPP),
    ] {
        std::fs::write(root.join(rel), content).unwrap();
    }

    carry_ok(&root, &["build"]);

    // I001 regression: conan writes CMakeUserPresets.json into the project
    // root; the scaffold .gitignore must cover it so a git-managed project
    // stays clean.
    assert!(root.join("CMakeUserPresets.json").is_file());
    let gitignore = std::fs::read_to_string(root.join(".gitignore")).unwrap();
    assert!(gitignore.contains("CMakeUserPresets.json"), "{gitignore}");

    let (stdout, _) = carry_ok(&root, &["run"]);
    assert!(stdout.contains("Hello from fmtapp!"), "{stdout}");
    assert!(stdout.contains("answer: 42"), "{stdout}");

    carry_ok(&root, &["build", "--release"]);
    let cache = std::fs::read_to_string(root.join("build/CMakeCache.txt")).unwrap();
    assert!(cache.contains("CMAKE_BUILD_TYPE:STRING=Release"), "{cache}");

    carry_ok(&root, &["build"]);
    let cache = std::fs::read_to_string(root.join("build/CMakeCache.txt")).unwrap();
    assert!(cache.contains("CMAKE_BUILD_TYPE:STRING=Debug"), "{cache}");

    let (stdout, _) = carry_ok(&root, &["test"]);
    assert!(stdout.contains("100% tests passed"), "{stdout}");

    // Dep removal path: remove fmt, revert the sources, still green.
    carry_ok(&root, &["remove", "fmt"]);
    for (rel, content) in [
        ("src/main.cpp", ORIG_MAIN_CPP),
        ("src/lib.cpp", ORIG_LIB_CPP),
        ("tests/test_main.cpp", ORIG_TEST_CPP),
    ] {
        std::fs::write(root.join(rel), content).unwrap();
    }
    carry_ok(&root, &["build"]);

    let cmake = std::fs::read_to_string(root.join("CMakeLists.txt")).unwrap();
    assert!(!cmake.contains("find_package(fmt"), "{cmake}");
    let conanfile = std::fs::read_to_string(root.join("conanfile.txt")).unwrap();
    assert!(!conanfile.contains("fmt"), "{conanfile}");

    // clean wipes build/ (and a second clean is a friendly no-op).
    let (stdout, _) = carry_ok(&root, &["clean"]);
    assert!(stdout.contains("removed build/"), "{stdout}");
    assert!(!root.join("build").exists());
    let (stdout, _) = carry_ok(&root, &["clean"]);
    assert!(stdout.contains("nothing to clean"), "{stdout}");
}

/// D12 regression (T009): a dependency whose recipe *validates*
/// `compiler.cppstd` (glaze/8.3.0 requires C++23) must build even when the
/// Conan profile's cppstd is older than the manifest's `cpp_std` (this
/// machine: gnu20 vs c++23). Before D12, `conan install` failed with
/// "Invalid: Current cppstd (gnu20) is lower than the required C++ standard
/// (23)"; carry now passes the project's standard to conan.
#[test]
#[ignore = "needs C++ toolchain + Conan Center network (D8)"]
fn glaze_full_loop() {
    let dir = tempfile::tempdir().unwrap();
    carry_ok(dir.path(), &["new", "glazeapp"]);
    let root = dir.path().join("glazeapp");

    std::fs::write(root.join("src/main.cpp"), GLAZE_MAIN_CPP).unwrap();

    // Bare `glaze` — the D11 "any" range; resolves to the highest available
    // version (8.3.0 at the time of writing, C++23-required).
    let (stdout, _) = carry_ok(&root, &["add", "glaze"]);
    assert!(stdout.contains("added glaze"), "{stdout}");

    carry_ok(&root, &["build"]);
    let (stdout, _) = carry_ok(&root, &["run"]);
    assert!(stdout.contains(r#""name":"Alice""#), "{stdout}");
    assert!(stdout.contains(r#""age":42"#), "{stdout}");
}
