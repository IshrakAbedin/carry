//! Build pipeline (DESIGN.md §6): `conan install` → cmake configure →
//! `cmake --build`. T005 adds the `run`/`test`/`clean` pipelines on top of
//! the same helpers.
//!
//! External commands stream their output straight to the terminal (never
//! buffered, §6); a failure becomes a carry error naming the tool, its exit
//! code, and the command line — no captured output, it was already streamed.

use std::io;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};

use crate::generate;
use crate::manifest::Manifest;
use crate::project::MANIFEST_NAME;

/// `carry build [--release]` (DESIGN.md §6, steps 1–5): load the manifest,
/// then `build_with`.
pub fn build(root: &Path, release: bool, verbose: bool) -> Result<()> {
    let manifest = Manifest::load(&root.join(MANIFEST_NAME))?;
    build_with(&manifest, root, release, verbose)
}

/// The pipeline (DESIGN.md §6, steps 1–5) with the manifest already loaded:
/// regenerate the generated files (write-if-changed), then conan install →
/// cmake configure → cmake --build, all in `root`. The pipeline is identical
/// with and without dependencies: conan writes the toolchain file either way.
fn build_with(manifest: &Manifest, root: &Path, release: bool, verbose: bool) -> Result<()> {
    generate::regenerate(manifest, root)?;

    let build_type = if release { "Release" } else { "Debug" };

    run(
        "conan",
        &[
            "install",
            ".",
            "--output-folder=build",
            "--build=missing",
            // `-s` takes the setting as a separate argument (not one token).
            "-s",
            &format!("build_type={build_type}"),
        ],
        root,
        verbose,
    )?;

    run(
        "cmake",
        &[
            "-S",
            ".",
            "-B",
            "build",
            "-G",
            "Ninja",
            "-DCMAKE_TOOLCHAIN_FILE=build/conan_toolchain.cmake",
            &format!("-DCMAKE_BUILD_TYPE={build_type}"),
            "-DCMAKE_C_COMPILER=gcc",
            "-DCMAKE_CXX_COMPILER=g++",
        ],
        root,
        verbose,
    )?;

    run("cmake", &["--build", "build"], root, verbose)
}

/// Run `cmd args...` in `cwd` with stdout/stderr inherited. Prints
/// `$ <cmd> <args...>` first when `verbose`. Missing tool →
/// "required tool not found on PATH"; non-zero exit → error with the exit
/// code and the full command line (DESIGN.md §6 tool invocation rules).
fn run(cmd: &str, args: &[&str], cwd: &Path, verbose: bool) -> Result<()> {
    let line = if args.is_empty() {
        cmd.to_string()
    } else {
        format!("{cmd} {}", args.join(" "))
    };
    if verbose {
        println!("$ {line}");
    }

    let mut child = match Command::new(cmd).args(args).current_dir(cwd).spawn() {
        Ok(child) => child,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            anyhow::bail!("required tool '{cmd}' not found on PATH");
        }
        Err(err) => return Err(err).with_context(|| format!("running {cmd}")),
    };
    let status = child.wait().with_context(|| format!("waiting for {cmd}"))?;
    if !status.success() {
        match status.code() {
            Some(code) => anyhow::bail!("{cmd} failed with exit code {code}\n  command: $ {line}"),
            None => anyhow::bail!("{cmd} terminated by a signal\n  command: $ {line}"),
        }
    }
    Ok(())
}

/// `carry run [args]...` (DESIGN.md §2/§6): build (Debug), then exec
/// `build/bin/<name> args...` with inherited stdio. The child's exit code is
/// propagated as carry's own exit code (DESIGN.md §8) — it is the program's
/// own exit, not a carry error; signal death has no code, so fall back to 101.
pub fn run_cmd(root: &Path, manifest: &Manifest, args: &[String], verbose: bool) -> Result<()> {
    build_with(manifest, root, false, verbose)?;
    let bin = root.join("build").join("bin").join(&manifest.package.name);
    if !bin.is_file() {
        anyhow::bail!(
            "executable not found: {}\n  help: check [package].type — only \"bin\" and \"both\" build an executable",
            bin.display()
        );
    }
    let line = if args.is_empty() {
        bin.display().to_string()
    } else {
        format!("{} {}", bin.display(), args.join(" "))
    };
    if verbose {
        println!("$ {line}");
    }
    let mut child = Command::new(&bin)
        .args(args)
        .current_dir(root)
        .spawn()
        .with_context(|| format!("running {}", bin.display()))?;
    let status = child.wait().with_context(|| "waiting for the program")?;
    if !status.success() {
        match status.code() {
            Some(code) => std::process::exit(code),
            None => std::process::exit(101),
        }
    }
    Ok(())
}

/// `carry test` (DESIGN.md §2/§6): build (Debug), then
/// `ctest --test-dir build --output-on-failure`. A ctest failure is a carry
/// error in the tool-failure format (like build failures), not a propagated
/// child exit code.
pub fn test_cmd(root: &Path, verbose: bool) -> Result<()> {
    build(root, false, verbose)?;
    run("ctest", &["--test-dir", "build", "--output-on-failure"], root, verbose)
}

/// `carry clean` (DESIGN.md §2): remove `root/build` recursively. If `build`
/// is absent, print "nothing to clean". A `build` symlink pointing outside
/// the project is refused (safety); a symlink pointing inside removes the
/// link itself, never the target.
pub fn clean_cmd(root: &Path) -> Result<()> {
    let build = root.join("build");
    let meta = match std::fs::symlink_metadata(&build) {
        Ok(meta) => meta,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            println!("nothing to clean");
            return Ok(());
        }
        Err(err) => return Err(err).with_context(|| format!("reading {}", build.display())),
    };
    if meta.file_type().is_symlink() {
        let target = std::fs::read_link(&build).with_context(|| "reading build/ symlink")?;
        // A relative symlink target resolves against the link's directory (root).
        let resolved = if target.is_absolute() { target } else { root.join(target) };
        match resolved.canonicalize() {
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                // Broken symlink: nothing to destroy, remove the link itself.
                std::fs::remove_file(&build)
                    .with_context(|| format!("removing {}", build.display()))?;
                println!("removed build/ (symlink)");
                return Ok(());
            }
            Ok(canon_target) => {
                let canon_root = root
                    .canonicalize()
                    .with_context(|| "resolving project root")?;
                if !canon_target.starts_with(&canon_root) {
                    anyhow::bail!(
                        "refusing to remove {}: symlink points outside the project ({})",
                        build.display(),
                        canon_target.display()
                    );
                }
            }
            Err(err) => {
                return Err(err)
                    .with_context(|| format!("resolving symlink target {}", resolved.display()))
            }
        }
        std::fs::remove_file(&build).with_context(|| format!("removing {}", build.display()))?;
        println!("removed build/ (symlink)");
        return Ok(());
    }
    std::fs::remove_dir_all(&build)
        .with_context(|| format!("removing {}", build.display()))?;
    println!("removed build/");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_reports_missing_tool() {
        let dir = tempfile::tempdir().unwrap();
        let err = run("carry-nonexistent-tool-xyz", &[], dir.path(), false).unwrap_err();
        assert_eq!(
            err.to_string(),
            "required tool 'carry-nonexistent-tool-xyz' not found on PATH"
        );
    }

    #[test]
    fn run_reports_nonzero_exit_with_command_line() {
        let dir = tempfile::tempdir().unwrap();
        let err = run("sh", &["-c", "exit 3"], dir.path(), false).unwrap_err();
        assert_eq!(
            err.to_string(),
            "sh failed with exit code 3\n  command: $ sh -c exit 3"
        );
    }

    #[test]
    fn clean_removes_build_dir() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("build")).unwrap();
        clean_cmd(dir.path()).unwrap();
        assert!(!dir.path().join("build").exists());
    }

    #[test]
    fn clean_without_build_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        clean_cmd(dir.path()).unwrap();
    }

    #[test]
    fn clean_refuses_symlink_pointing_outside_project() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("victim");
        std::fs::create_dir_all(&victim).unwrap();
        std::os::unix::fs::symlink(&victim, dir.path().join("build")).unwrap();
        let err = clean_cmd(dir.path()).unwrap_err();
        assert!(
            err.to_string().contains("refusing to remove"),
            "unexpected error: {err}"
        );
        assert!(victim.exists(), "symlink target must survive");
        assert!(
            dir.path().join("build").exists(),
            "the link itself must survive"
        );
    }

    #[test]
    fn clean_symlink_inside_project_removes_link_only() {
        let dir = tempfile::tempdir().unwrap();
        let inside = dir.path().join("real-build");
        std::fs::create_dir_all(&inside).unwrap();
        std::os::unix::fs::symlink(&inside, dir.path().join("build")).unwrap();
        clean_cmd(dir.path()).unwrap();
        assert!(!dir.path().join("build").exists(), "link must be removed");
        assert!(inside.exists(), "target must survive");
    }
}
