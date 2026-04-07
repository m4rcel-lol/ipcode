//! CLI argument parsing for the `ipcode` binary.

use clap::{Parser, Subcommand};

/// IPcode — a programming language where every line is a fake IPv4 address.
#[derive(Parser, Debug)]
#[command(
    name = "ipcode",
    version = env!("CARGO_PKG_VERSION"),
    about = "IPcode interpreter and toolchain",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Available `ipcode` sub-commands.
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Parse and execute an IPcode program.
    Run {
        /// Path to the `.ipc` source file.
        file: String,
        /// Override the default cycle limit (default: 10 000 000).
        #[arg(long, default_value_t = 10_000_000)]
        cycle_limit: usize,
    },
    /// Validate syntax without running the program.
    Check {
        /// Path to the `.ipc` source file.
        file: String,
    },
    /// Print a human-readable disassembly of every instruction.
    Disasm {
        /// Path to the `.ipc` source file.
        file: String,
    },
    /// Run in step-through debug mode, printing register state after every instruction.
    Debug {
        /// Path to the `.ipc` source file.
        file: String,
        /// Override the default cycle limit (default: 10 000 000).
        #[arg(long, default_value_t = 10_000_000)]
        cycle_limit: usize,
    },
    /// Print IPcode version.
    Version,
}
