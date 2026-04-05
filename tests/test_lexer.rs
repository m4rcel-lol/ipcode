//! Integration tests for the IPcode lexer.

use ipcode::errors::ErrorKind;
use ipcode::lexer::lex;

#[test]
fn test_lex_single_instruction() {
    let tokens = lex("1.0.0.42\n", "t.ipc").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].a, 1);
    assert_eq!(tokens[0].b, 0);
    assert_eq!(tokens[0].c, 0);
    assert_eq!(tokens[0].d, 42);
}

#[test]
fn test_lex_comment_only() {
    let tokens = lex("# just a comment\n", "t.ipc").unwrap();
    assert!(tokens.is_empty());
}

#[test]
fn test_lex_blank_lines_skipped() {
    let tokens = lex("\n\n1.0.0.1\n\n", "t.ipc").unwrap();
    assert_eq!(tokens.len(), 1);
}

#[test]
fn test_lex_inline_comment_stripped() {
    let tokens = lex("2.0.0.1  # ADD R0, R1\n", "t.ipc").unwrap();
    assert_eq!(tokens[0].a, 2);
    assert_eq!(tokens[0].b, 0);
    assert_eq!(tokens[0].d, 1);
}

#[test]
fn test_lex_multiple_instructions() {
    let src = "1.0.0.40\n1.0.1.2\n2.0.0.1\n";
    let tokens = lex(src, "t.ipc").unwrap();
    assert_eq!(tokens.len(), 3);
}

#[test]
fn test_lex_instruction_line_numbering() {
    let src = "# skip\n\n1.0.0.1\n# skip\n1.0.1.2\n";
    let tokens = lex(src, "t.ipc").unwrap();
    assert_eq!(tokens[0].instruction_line, 1);
    assert_eq!(tokens[1].instruction_line, 2);
}

#[test]
fn test_lex_source_line_numbering() {
    let src = "# skip\n\n1.0.0.1\n";
    let tokens = lex(src, "t.ipc").unwrap();
    assert_eq!(tokens[0].source_line, 3);
}

#[test]
fn test_lex_max_octets() {
    let tokens = lex("255.255.255.255\n", "t.ipc").unwrap();
    assert_eq!(tokens[0].a, 255);
    assert_eq!(tokens[0].b, 255);
    assert_eq!(tokens[0].c, 255);
    assert_eq!(tokens[0].d, 255);
}

#[test]
fn test_lex_zero_octets() {
    let tokens = lex("0.0.0.0\n", "t.ipc").unwrap();
    assert_eq!(tokens[0].a, 0);
    assert_eq!(tokens[0].d, 0);
}

#[test]
fn test_lex_error_too_few_octets() {
    let err = lex("1.0.0\n", "t.ipc").unwrap_err();
    assert_eq!(err.kind, ErrorKind::InvalidFormat);
}

#[test]
fn test_lex_error_octet_out_of_range() {
    let err = lex("1.0.0.256\n", "t.ipc").unwrap_err();
    assert_eq!(err.kind, ErrorKind::InvalidFormat);
}

#[test]
fn test_lex_error_non_numeric_octet() {
    let err = lex("1.X.0.0\n", "t.ipc").unwrap_err();
    assert_eq!(err.kind, ErrorKind::InvalidFormat);
}

#[test]
fn test_lex_error_extra_parts() {
    // splitn(4, '.') will capture everything after the 3rd dot in the 4th part.
    // "1.2.3.4.5" → parts = ["1","2","3","4.5"] → d fails to parse as u8
    let err = lex("1.2.3.4.5\n", "t.ipc").unwrap_err();
    assert_eq!(err.kind, ErrorKind::InvalidFormat);
}
