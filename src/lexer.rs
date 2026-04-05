//! Lexer for IPcode — tokenizes `.ipc` source lines into raw `(A, B, C, D)` tuples.
//!
//! Each non-comment, non-blank line must be an IPv4-style address `A.B.C.D`
//! where every octet is an integer in 0–255.

use crate::errors::{ErrorKind, IpcError};

/// A raw decoded IP-address token with its source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// First octet — opcode category.
    pub a: u8,
    /// Second octet — sub-opcode / primary operand.
    pub b: u8,
    /// Third octet — second operand.
    pub c: u8,
    /// Fourth octet — modifier / third operand.
    pub d: u8,
    /// 1-based instruction index (comments and blank lines excluded).
    pub instruction_line: usize,
    /// 1-based source file line number.
    pub source_line: usize,
}

/// Lex a `.ipc` source string into a list of [`Token`]s.
///
/// # Errors
/// Returns an [`IpcError`] if any non-comment, non-blank line is not a valid
/// IPv4-format address with all four octets in range 0–255.
pub fn lex(source: &str, file: &str) -> Result<Vec<Token>, IpcError> {
    let mut tokens = Vec::new();
    let mut instruction_line = 0usize;

    for (idx, raw_line) in source.lines().enumerate() {
        let source_line = idx + 1;
        // Strip inline comments and whitespace.
        let line = match raw_line.find('#') {
            Some(pos) => raw_line[..pos].trim(),
            None => raw_line.trim(),
        };

        if line.is_empty() {
            continue;
        }

        instruction_line += 1;
        let token = parse_address(line, file, source_line, instruction_line)?;
        tokens.push(token);
    }

    Ok(tokens)
}

/// Parse a single address string `"A.B.C.D"` into a [`Token`].
fn parse_address(
    addr: &str,
    file: &str,
    source_line: usize,
    instruction_line: usize,
) -> Result<Token, IpcError> {
    let parts: Vec<&str> = addr.splitn(4, '.').collect();
    if parts.len() != 4 {
        return Err(IpcError::new(
            ErrorKind::InvalidFormat,
            file,
            source_line,
            Some(addr.to_string()),
            format!(
                "expected 4 octets separated by '.', got {}",
                parts.len()
            ),
        ));
    }

    let mut octets = [0u8; 4];
    for (i, part) in parts.iter().enumerate() {
        let n: u32 = part.parse().map_err(|_| {
            IpcError::new(
                ErrorKind::InvalidFormat,
                file,
                source_line,
                Some(addr.to_string()),
                format!("octet {} ('{}') is not a valid integer", i + 1, part),
            )
        })?;
        if n > 255 {
            return Err(IpcError::new(
                ErrorKind::InvalidFormat,
                file,
                source_line,
                Some(addr.to_string()),
                format!("octet {} value {} is out of range 0–255", i + 1, n),
            ));
        }
        octets[i] = n as u8;
    }

    Ok(Token {
        a: octets[0],
        b: octets[1],
        c: octets[2],
        d: octets[3],
        instruction_line,
        source_line,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_lex() {
        let src = "1.0.0.42\n";
        let tokens = lex(src, "test.ipc").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].a, 1);
        assert_eq!(tokens[0].b, 0);
        assert_eq!(tokens[0].c, 0);
        assert_eq!(tokens[0].d, 42);
        assert_eq!(tokens[0].instruction_line, 1);
    }

    #[test]
    fn test_comment_skipped() {
        let src = "# This is a comment\n1.0.0.1\n";
        let tokens = lex(src, "test.ipc").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].instruction_line, 1);
    }

    #[test]
    fn test_inline_comment() {
        let src = "1.0.0.72  # LOAD R0, 'H'\n";
        let tokens = lex(src, "test.ipc").unwrap();
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].d, 72);
    }

    #[test]
    fn test_blank_line_skipped() {
        let src = "\n1.0.0.1\n\n1.0.1.2\n";
        let tokens = lex(src, "test.ipc").unwrap();
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].instruction_line, 1);
        assert_eq!(tokens[1].instruction_line, 2);
    }

    #[test]
    fn test_invalid_format_three_parts() {
        let src = "1.0.0\n";
        let err = lex(src, "test.ipc").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidFormat);
    }

    #[test]
    fn test_invalid_octet_out_of_range() {
        let src = "1.0.0.256\n";
        let err = lex(src, "test.ipc").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidFormat);
    }

    #[test]
    fn test_invalid_octet_non_numeric() {
        let src = "1.0.X.0\n";
        let err = lex(src, "test.ipc").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidFormat);
    }

    #[test]
    fn test_max_values() {
        let src = "255.255.255.255\n";
        let tokens = lex(src, "test.ipc").unwrap();
        assert_eq!(tokens[0].a, 255);
        assert_eq!(tokens[0].d, 255);
    }

    #[test]
    fn test_instruction_line_numbering() {
        let src = "# comment\n\n1.0.0.1\n# another\n1.0.1.2\n";
        let tokens = lex(src, "test.ipc").unwrap();
        assert_eq!(tokens[0].instruction_line, 1);
        assert_eq!(tokens[1].instruction_line, 2);
    }
}
