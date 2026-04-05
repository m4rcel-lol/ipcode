//! Custom error types for IPcode with file/line context.

use std::fmt;

/// Represents an error that occurred during IPcode execution or parsing.
#[derive(Debug)]
pub struct IpcError {
    /// The source file name.
    pub file: String,
    /// The 1-based line number.
    pub line: usize,
    /// The raw IP address string that caused the error, if any.
    pub raw: Option<String>,
    /// Human-readable description of the error.
    pub message: String,
    /// The kind of error.
    pub kind: ErrorKind,
}

/// Categorizes the kinds of errors IPcode can produce.
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorKind {
    /// IP address format is invalid (not four octets, octet out of range).
    InvalidFormat,
    /// The opcode is not recognized.
    UnknownOpcode,
    /// A register index is out of the valid range (0–15).
    InvalidRegister,
    /// Division by zero was attempted.
    DivisionByZero,
    /// The data stack overflowed.
    StackOverflow,
    /// The data stack underflowed (pop on empty stack).
    StackUnderflow,
    /// An invalid memory address was accessed.
    MemoryViolation,
    /// A jump targeted a non-existent line.
    InvalidJump,
    /// The maximum cycle count was exceeded.
    CycleLimitExceeded,
    /// The call stack overflowed.
    CallStackOverflow,
    /// The call stack underflowed (RET with empty call stack).
    CallStackUnderflow,
    /// A general I/O error occurred.
    IoError,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            ErrorKind::InvalidFormat => "InvalidFormat",
            ErrorKind::UnknownOpcode => "UnknownOpcode",
            ErrorKind::InvalidRegister => "InvalidRegister",
            ErrorKind::DivisionByZero => "DivisionByZero",
            ErrorKind::StackOverflow => "StackOverflow",
            ErrorKind::StackUnderflow => "StackUnderflow",
            ErrorKind::MemoryViolation => "MemoryViolation",
            ErrorKind::InvalidJump => "InvalidJump",
            ErrorKind::CycleLimitExceeded => "CycleLimitExceeded",
            ErrorKind::CallStackOverflow => "CallStackOverflow",
            ErrorKind::CallStackUnderflow => "CallStackUnderflow",
            ErrorKind::IoError => "IoError",
        };
        write!(f, "{}", s)
    }
}

impl fmt::Display for IpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(raw) = &self.raw {
            write!(
                f,
                "[{}] error at {}:{} ({}): {}",
                self.kind, self.file, self.line, raw, self.message
            )
        } else {
            write!(
                f,
                "[{}] error at {}:{}: {}",
                self.kind, self.file, self.line, self.message
            )
        }
    }
}

impl std::error::Error for IpcError {}

impl IpcError {
    /// Create a new `IpcError`.
    pub fn new(
        kind: ErrorKind,
        file: impl Into<String>,
        line: usize,
        raw: Option<String>,
        message: impl Into<String>,
    ) -> Self {
        IpcError {
            file: file.into(),
            line,
            raw,
            message: message.into(),
            kind,
        }
    }
}
