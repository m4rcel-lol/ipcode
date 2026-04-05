//! Integration tests for the IPcode parser.

use ipcode::errors::ErrorKind;
use ipcode::instructions::Instruction;
use ipcode::lexer::lex;
use ipcode::parser::parse;

fn parse_src(src: &str) -> Result<Vec<Instruction>, ipcode::errors::IpcError> {
    let tokens = lex(src, "t.ipc").unwrap();
    parse(&tokens, "t.ipc")
}

// ── Category 0 ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_nop() {
    assert_eq!(parse_src("0.0.0.0\n").unwrap()[0], Instruction::Nop);
}

#[test]
fn test_parse_halt() {
    assert_eq!(parse_src("0.1.0.0\n").unwrap()[0], Instruction::Halt);
}

#[test]
fn test_parse_syscall() {
    assert_eq!(
        parse_src("0.2.3.0\n").unwrap()[0],
        Instruction::Syscall(3)
    );
}

// ── Category 1 ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_load() {
    assert_eq!(
        parse_src("1.0.2.99\n").unwrap()[0],
        Instruction::Load(2, 99)
    );
}

#[test]
fn test_parse_loadhi() {
    assert_eq!(
        parse_src("1.6.0.100\n").unwrap()[0],
        Instruction::Loadhi(0, 100)
    );
}

#[test]
fn test_parse_mov() {
    assert_eq!(
        parse_src("1.3.0.1\n").unwrap()[0],
        Instruction::Mov(0, 1)
    );
}

#[test]
fn test_parse_push_pop() {
    let instrs = parse_src("1.4.3.0\n1.5.4.0\n").unwrap();
    assert_eq!(instrs[0], Instruction::Push(3));
    assert_eq!(instrs[1], Instruction::Pop(4));
}

// ── Category 2 ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_arithmetic() {
    assert_eq!(parse_src("2.0.0.1\n").unwrap()[0], Instruction::Add(0, 1));
    assert_eq!(parse_src("2.1.0.1\n").unwrap()[0], Instruction::Sub(0, 1));
    assert_eq!(parse_src("2.2.0.1\n").unwrap()[0], Instruction::Mul(0, 1));
    assert_eq!(parse_src("2.3.0.1\n").unwrap()[0], Instruction::Div(0, 1));
    assert_eq!(parse_src("2.4.0.1\n").unwrap()[0], Instruction::Mod(0, 1));
    assert_eq!(parse_src("2.5.0.0\n").unwrap()[0], Instruction::Inc(0));
    assert_eq!(parse_src("2.6.0.0\n").unwrap()[0], Instruction::Dec(0));
}

// ── Category 4 ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_comparison() {
    assert_eq!(parse_src("4.0.0.1\n").unwrap()[0], Instruction::Cmp(0, 1));
    assert_eq!(parse_src("4.1.0.1\n").unwrap()[0], Instruction::Eq(0, 1));
    assert_eq!(parse_src("4.6.0.1\n").unwrap()[0], Instruction::Gte(0, 1));
}

// ── Category 5 ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_jumps() {
    assert_eq!(parse_src("5.0.0.5\n").unwrap()[0], Instruction::Jmp(5));
    assert_eq!(parse_src("5.1.0.5\n").unwrap()[0], Instruction::Jt(5));
    assert_eq!(parse_src("5.2.0.5\n").unwrap()[0], Instruction::Jf(5));
}

// ── Category 6 ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_io() {
    assert_eq!(parse_src("6.0.2.0\n").unwrap()[0], Instruction::Printi(2));
    assert_eq!(parse_src("6.1.0.0\n").unwrap()[0], Instruction::Printc(0));
    assert_eq!(parse_src("6.3.0.0\n").unwrap()[0], Instruction::Println);
}

// ── Category 7 ──────────────────────────────────────────────────────────────

#[test]
fn test_parse_call_ret() {
    assert_eq!(parse_src("7.0.0.5\n").unwrap()[0], Instruction::Call(5));
    assert_eq!(parse_src("7.1.0.0\n").unwrap()[0], Instruction::Ret);
}

// ── Errors ───────────────────────────────────────────────────────────────────

#[test]
fn test_parse_invalid_register() {
    let tokens = lex("1.0.16.0\n", "t.ipc").unwrap();
    let err = parse(&tokens, "t.ipc").unwrap_err();
    assert_eq!(err.kind, ErrorKind::InvalidRegister);
}

#[test]
fn test_parse_unknown_opcode_category() {
    let tokens = lex("200.0.0.0\n", "t.ipc").unwrap();
    let err = parse(&tokens, "t.ipc").unwrap_err();
    assert_eq!(err.kind, ErrorKind::UnknownOpcode);
}

#[test]
fn test_parse_unknown_sub_opcode() {
    let tokens = lex("0.9.0.0\n", "t.ipc").unwrap();
    let err = parse(&tokens, "t.ipc").unwrap_err();
    assert_eq!(err.kind, ErrorKind::UnknownOpcode);
}
