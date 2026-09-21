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
/// regenerate the generated files (write-if-changed), then conan install →
/// cmake configure → cmake --build, all in `root`. The pipeline is identical
/// with and without dependencies: conan writes the toolchain file either way.
pub fn build(root: &Path, release: bool, verbose: bool) -> Result<()> {
    let manifest = Manifest::load(&root.join(MANIFEST_NAME))?;
    generate::regenerate(&manifest, root)?;

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
}
