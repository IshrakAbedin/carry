//! carry — a C/C++ project manager in the spirit of Cargo.
//!
//! T001: CLI scaffold. All MVP subcommands from DESIGN.md §2 are defined; each
//! command is a stub returning "not implemented" until its owning task lands
//! (new/init → T002, build → T004, run/test/clean → T005, add/remove → T006).

// Module files are empty scaffolds until their owning tasks fill them in;
// remove the allow on each as it becomes used (T002–T005).
#[allow(dead_code)]
mod build;
#[allow(dead_code)]
mod generate;
#[allow(dead_code)]
mod manifest;
#[allow(dead_code)]
mod project;
#[allow(dead_code)]
mod template;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "carry", version, about = "A C/C++ project manager in the spirit of Cargo")]
struct Cli {
    /// Print each external command before running it (wired up in T004)
    #[arg(long, global = true)]
    #[allow(dead_code)]
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
    let not_implemented = |cmd: &str, planned: &str| -> ! {
        eprintln!("error: {cmd} not implemented yet (planned in {planned})");
        std::process::exit(1);
    };
    match cli.command {
        Commands::New { .. } => not_implemented("new", "T002"),
        Commands::Init => not_implemented("init", "T002"),
        Commands::Build { .. } => not_implemented("build", "T004"),
        Commands::Run { .. } => not_implemented("run", "T005"),
        Commands::Test => not_implemented("test", "T005"),
        Commands::Clean => not_implemented("clean", "T005"),
        Commands::Add { .. } => not_implemented("add", "T006"),
        Commands::Remove { .. } => not_implemented("remove", "T006"),
    }
}
