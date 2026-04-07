//! `ipcode` — the IPcode interpreter binary.
//!
//! Every line of an `.ipc` program is a fake IPv4 address `A.B.C.D`.

use clap::Parser;
use colored::Colorize;
use ipcode::cli::{Cli, Command};
use ipcode::errors::{ErrorKind, IpcError};
use ipcode::instructions::Instruction;
use ipcode::{lexer, parser, vm};

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Version => {
            println!("ipcode {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Run { file, cycle_limit } => run_file(&file, cycle_limit),
        Command::Check { file } => check_file(&file),
        Command::Disasm { file } => disasm_file(&file),
        Command::Debug { file, cycle_limit } => debug_file(&file, cycle_limit),
    };

    if let Err(e) = result {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

/// Load and compile an `.ipc` source file into a list of instructions.
fn load_program(path: &str) -> Result<Vec<Instruction>, IpcError> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        IpcError::new(
            ErrorKind::IoError,
            path,
            0,
            None,
            format!("cannot read file '{}': {}", path, e),
        )
    })?;
    let tokens = lexer::lex(&source, path)?;
    parser::parse(&tokens, path)
}

/// Execute an `.ipc` program.
fn run_file(path: &str, cycle_limit: usize) -> Result<(), IpcError> {
    let program = load_program(path)?;
    let mut machine = vm::Vm::new(path).with_cycle_limit(cycle_limit);
    machine.run_program(&program)?;
    Ok(())
}

/// Validate an `.ipc` program without executing it.
fn check_file(path: &str) -> Result<(), IpcError> {
    let program = load_program(path)?;
    println!(
        "{} '{}' — {} instruction(s), no syntax errors.",
        "OK".green().bold(),
        path,
        program.len()
    );
    Ok(())
}

/// Disassemble an `.ipc` program to human-readable output.
fn disasm_file(path: &str) -> Result<(), IpcError> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        IpcError::new(
            ErrorKind::IoError,
            path,
            0,
            None,
            format!("cannot read file '{}': {}", path, e),
        )
    })?;
    let tokens = lexer::lex(&source, path)?;
    let program = parser::parse(&tokens, path)?;

    println!("{}", format!("Disassembly of '{}':", path).cyan().bold());
    println!(
        "{:<6} {:<18} {:<10} {}",
        "Line".bold(),
        "Address".bold(),
        "Instr".bold(),
        "Operands".bold()
    );
    println!("{}", "─".repeat(60).dimmed());

    for (i, (token, instr)) in tokens.iter().zip(program.iter()).enumerate() {
        let addr = format!("{}.{}.{}.{}", token.a, token.b, token.c, token.d);
        println!(
            "{:<6} {:<18} {:<10} {}",
            format!("{:>4}", i + 1).cyan(),
            addr.yellow(),
            instr.mnemonic().green().bold(),
            instr.operands()
        );
    }

    Ok(())
}

/// Execute an `.ipc` program in step-through debug mode.
fn debug_file(path: &str, cycle_limit: usize) -> Result<(), IpcError> {
    let program = load_program(path)?;
    let mut machine = vm::Vm::new(path).with_cycle_limit(cycle_limit);
    machine.run_program_debug(&program)?;
    Ok(())
}
