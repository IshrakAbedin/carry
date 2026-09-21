//! carry — a C/C++ project manager in the spirit of Cargo.
//!
//! T001: CLI scaffold. T002: `new`/`init` (manifest + scaffold).
//! T003: generator wired in — `build`/`run`/`test` first locate the project,
//! load the manifest, and regenerate the generated files (write-if-changed).
//! T004: `build` runs the real pipeline (conan → cmake → ninja); `run`/`test`
//! still report "not implemented" until T005 (clean → T005, add/remove → T006).

mod build;
mod generate;
mod manifest;
mod project;
mod template;

use crate::manifest::Manifest;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "carry", version, about = "A C/C++ project manager in the spirit of Cargo")]
struct Cli {
    /// Print each external command before running it
    #[arg(long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new carry project at ./<name>/
    New {
        /// Project name
        name: String,

        /// Also add a library target
        #[arg(long)]
        lib: bool,
    },

    /// Turn the current directory into a carry project
    Init,

    /// Regenerate files, install dependencies, configure, and build
    Build {
        /// Build with the Release build type (default: Debug)
        #[arg(long)]
        release: bool,
    },

    /// Build (Debug) then run the project binary with the given arguments
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Build (Debug) then run the test suite
    Test,

    /// Remove the build/ directory
    Clean,

    /// Add or update a dependency (e.g. `fmt` or `fmt/10.2.1`)
    Add {
        /// Package spec
        pkg: String,
    },

    /// Remove a dependency
    Remove {
        /// Package name
        pkg: String,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    let Cli { verbose, command } = cli;
    let not_implemented = |cmd: &str, planned: &str| -> anyhow::Error {
        anyhow::anyhow!("{cmd} not implemented yet (planned in {planned})")
    };
    match command {
        Commands::New { name, lib } => template::cmd_new(&name, lib),
        Commands::Init => template::cmd_init(),
        Commands::Build { release } => {
            let root = project::find_project_root()?;
            build::build(&root, release, verbose)
        }
        Commands::Run { .. } => {
            prepare_project()?;
            Err(not_implemented("run", "T005"))
        }
        Commands::Test => {
            prepare_project()?;
            Err(not_implemented("test", "T005"))
        }
        Commands::Clean => Err(not_implemented("clean", "T005")),
        Commands::Add { .. } => Err(not_implemented("add", "T006")),
        Commands::Remove { .. } => Err(not_implemented("remove", "T006")),
    }
}

/// Shared first step for `build`/`run`/`test` (DESIGN.md §6, steps 1–2):
/// locate the project root, load the manifest, and regenerate the generated
/// files (write-if-changed).
fn prepare_project() -> anyhow::Result<()> {
    let root = project::find_project_root()?;
    let manifest = Manifest::load(&root.join(project::MANIFEST_NAME))?;
    let _ = generate::regenerate(&manifest, &root)?;
    Ok(())
}
