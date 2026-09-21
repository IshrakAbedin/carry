//! carry — a C/C++ project manager in the spirit of Cargo.
//!
//! T001: CLI scaffold. T002: `new`/`init` (manifest + scaffold).
//! T003: generator wired in — `build`/`run`/`test` first locate the project,
//! load the manifest, and regenerate the generated files (write-if-changed).
//! T004: `build` runs the real pipeline (conan → cmake → ninja).
//! T005: `run` (build Debug + exec, child exit code propagated), `test`
//! (build Debug + ctest), `clean` (remove build/).
//! T006: `add`/`remove` (CARRY.toml dependency edits, format-preserving).

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
    #[command(long_about = "Build (Debug), then run the project binary with the given arguments.\n\n\
        Carry flags must come before user arguments; separate them with `--` when the program \
        itself takes flags:\n\n    carry run -- --release")]
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Build (Debug) then run the test suite
    Test,

    /// Remove the build/ directory
    Clean,

    /// Add or update a dependency in CARRY.toml (e.g. `fmt` or `fmt@10.2.1`)
    Add {
        /// Package: `pkg` or `pkg@version` (version may be a range, e.g. `fmt@>=10`)
        pkg: String,
    },

    /// Remove a dependency from CARRY.toml
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
    match command {
        Commands::New { name, lib } => template::cmd_new(&name, lib),
        Commands::Init => template::cmd_init(),
        Commands::Build { release } => {
            let root = project::find_project_root()?;
            build::build(&root, release, verbose)
        }
        Commands::Run { args } => {
            let root = project::find_project_root()?;
            let manifest = Manifest::load(&root.join(project::MANIFEST_NAME))?;
            build::run_cmd(&root, &manifest, &args, verbose)
        }
        Commands::Test => {
            let root = project::find_project_root()?;
            build::test_cmd(&root, verbose)
        }
        Commands::Clean => {
            let root = project::find_project_root()?;
            build::clean_cmd(&root)
        }
        Commands::Add { pkg } => {
            let root = project::find_project_root()?;
            let path = root.join(project::MANIFEST_NAME);
            let (name, version) = manifest::parse_dep_spec(&pkg)?;
            let mut m = Manifest::load(&path)?;
            m.add_dep(&path, &name, &version)
        }
        Commands::Remove { pkg } => {
            let root = project::find_project_root()?;
            let path = root.join(project::MANIFEST_NAME);
            let mut m = Manifest::load(&path)?;
            m.remove_dep(&path, &pkg)
        }
    }
}
