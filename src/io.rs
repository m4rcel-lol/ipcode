//! I/O helpers for IPcode — wraps stdin/stdout operations used by the VM.

use crate::errors::{ErrorKind, IpcError};
use std::io::{self, BufRead, Write};

/// Read a line from stdin and parse it as an `i64`.
///
/// # Errors
/// Returns an [`IoError`](ErrorKind::IoError) if the line cannot be read or
/// parsed as an integer.
pub fn read_integer(file: &str, line: usize) -> Result<i64, IpcError> {
    let stdin = io::stdin();
    let mut input = String::new();
    io::stdout().flush().ok();
    stdin.lock().read_line(&mut input).map_err(|e| {
        IpcError::new(
            ErrorKind::IoError,
            file,
            line,
            None,
            format!("failed to read from stdin: {}", e),
        )
    })?;
    input.trim().parse::<i64>().map_err(|_| {
        IpcError::new(
            ErrorKind::IoError,
            file,
            line,
            None,
            format!("'{}' is not a valid integer", input.trim()),
        )
    })
}

/// Read a single character from stdin and return its ASCII code as `i64`.
///
/// # Errors
/// Returns an [`IoError`](ErrorKind::IoError) if no character can be read.
pub fn read_char(file: &str, line: usize) -> Result<i64, IpcError> {
    let stdin = io::stdin();
    let mut input = String::new();
    io::stdout().flush().ok();
    stdin.lock().read_line(&mut input).map_err(|e| {
        IpcError::new(
            ErrorKind::IoError,
            file,
            line,
            None,
            format!("failed to read from stdin: {}", e),
        )
    })?;
    input.chars().next().map(|c| c as i64).ok_or_else(|| {
        IpcError::new(
            ErrorKind::IoError,
            file,
            line,
            None,
            "stdin returned empty input",
        )
    })
}

/// Print an integer to stdout without a trailing newline.
pub fn print_integer(value: i64) {
    print!("{}", value);
    io::stdout().flush().ok();
}

/// Print an ASCII character to stdout without a trailing newline.
///
/// If `value` is not in the printable ASCII range this will still output the
/// corresponding Unicode scalar, mirroring `char::from_u32`.
pub fn print_char(value: i64) {
    let c = char::from_u32(value as u32).unwrap_or('?');
    print!("{}", c);
    io::stdout().flush().ok();
}

/// Print a newline to stdout.
pub fn print_newline() {
    println!();
}
