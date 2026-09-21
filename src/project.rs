//! Project discovery: find the carry project root (walk up for `CARRY.toml`).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Manifest file name (DESIGN.md §4) — its presence marks the project root.
pub const MANIFEST_NAME: &str = "CARRY.toml";

/// Walk up from the current directory (inclusive) to the nearest ancestor
/// that contains `CARRY.toml`. Errors if none is found.
pub fn find_project_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().context("reading current directory")?;
    root_from(&cwd)
        .ok_or_else(|| anyhow::anyhow!("not a carry project (no CARRY.toml found)"))
}

fn root_from(start: &Path) -> Option<PathBuf> {
    let mut dir: Option<&Path> = Some(start);
    while let Some(d) = dir {
        if d.join(MANIFEST_NAME).is_file() {
            return Some(d.to_path_buf());
        }
        dir = d.parent();
    }
    None
}
