//! Integration tests for the IPcode virtual machine.

use ipcode::errors::ErrorKind;
use ipcode::lexer::lex;
use ipcode::parser::parse;
use ipcode::vm::Vm;

/// Helper: compile and run an IPcode snippet, returning the final VM state.
#[allow(dead_code)]
fn run_src(src: &str) -> ipcode::vm::VmState {
    let tokens = lex(src, "t.ipc").unwrap();
    let program = parse(&tokens, "t.ipc").unwrap();
    let mut vm = Vm::new("t.ipc");
    vm.run_program(&program).unwrap()
}

/// Helper: compile and run, then return the 16 registers and flags.
fn run_and_regs(src: &str) -> ([i64; 16], bool) {
    let tokens = lex(src, "t.ipc").unwrap();
    let program = parse(&tokens, "t.ipc").unwrap();
    let mut vm = Vm::new("t.ipc");
    vm.run_program(&program).unwrap();
    (*vm.registers(), vm.flags())
}

// ── Category 0 ──────────────────────────────────────────────────────────────

#[test]
fn test_nop_does_nothing() {
    let (regs, _) = run_and_regs("0.0.0.0\n0.1.0.0\n");
    assert_eq!(regs[0], 0);
}

#[test]
fn test_halt_stops_early() {
    // Only first LOAD should execute.
    let (regs, _) = run_and_regs("1.0.0.7\n0.1.0.0\n1.0.0.99\n");
    assert_eq!(regs[0], 7);
}

// ── Category 1 ──────────────────────────────────────────────────────────────

#[test]
fn test_load() {
    let (regs, _) = run_and_regs("1.0.3.42\n0.1.0.0\n");
    assert_eq!(regs[3], 42);
}

#[test]
fn test_loadhi() {
    // R0 = 1, then LOADHI R0, 2 → R0 = (2 << 8) | 1 = 513
    let (regs, _) = run_and_regs("1.0.0.1\n1.6.0.2\n0.1.0.0\n");
    assert_eq!(regs[0], (2 << 8) | 1);
}

#[test]
fn test_mov() {
    let (regs, _) = run_and_regs("1.0.0.55\n1.3.1.0\n0.1.0.0\n");
    assert_eq!(regs[0], 55);
    assert_eq!(regs[1], 55);
}

#[test]
fn test_push_pop() {
    // LOAD R0=5, PUSH R0, LOAD R0=0, POP R1 → R1=5
    let (regs, _) = run_and_regs("1.0.0.5\n1.4.0.0\n1.0.0.0\n1.5.1.0\n0.1.0.0\n");
    assert_eq!(regs[1], 5);
}

// ── Category 2 ──────────────────────────────────────────────────────────────

#[test]
fn test_add() {
    let (regs, _) = run_and_regs("1.0.0.40\n1.0.1.2\n2.0.0.1\n0.1.0.0\n");
    assert_eq!(regs[0], 42);
}

#[test]
fn test_sub() {
    let (regs, _) = run_and_regs("1.0.0.10\n1.0.1.3\n2.1.0.1\n0.1.0.0\n");
    assert_eq!(regs[0], 7);
}

#[test]
fn test_mul() {
    let (regs, _) = run_and_regs("1.0.0.6\n1.0.1.7\n2.2.0.1\n0.1.0.0\n");
    assert_eq!(regs[0], 42);
}

#[test]
fn test_div() {
    let (regs, _) = run_and_regs("1.0.0.20\n1.0.1.4\n2.3.0.1\n0.1.0.0\n");
    assert_eq!(regs[0], 5);
}

#[test]
fn test_mod() {
    let (regs, _) = run_and_regs("1.0.0.10\n1.0.1.3\n2.4.0.1\n0.1.0.0\n");
    assert_eq!(regs[0], 1);
}

#[test]
fn test_inc() {
    let (regs, _) = run_and_regs("1.0.0.41\n2.5.0.0\n0.1.0.0\n");
    assert_eq!(regs[0], 42);
}

#[test]
fn test_dec() {
    let (regs, _) = run_and_regs("1.0.0.43\n2.6.0.0\n0.1.0.0\n");
    assert_eq!(regs[0], 42);
}

#[test]
fn test_neg() {
    let (regs, _) = run_and_regs("1.0.0.5\n2.7.1.0\n0.1.0.0\n");
    assert_eq!(regs[1], -5);
}

#[test]
fn test_div_by_zero() {
    let tokens = lex("1.0.0.5\n1.0.1.0\n2.3.0.1\n", "t.ipc").unwrap();
    let program = parse(&tokens, "t.ipc").unwrap();
    let mut vm = Vm::new("t.ipc");
    let err = vm.run_program(&program).unwrap_err();
    assert_eq!(err.kind, ErrorKind::DivisionByZero);
}

// ── Category 3 ──────────────────────────────────────────────────────────────

#[test]
fn test_and() {
    let (regs, _) = run_and_regs("1.0.0.12\n1.0.1.10\n3.0.0.1\n0.1.0.0\n");
    assert_eq!(regs[0], 12 & 10);
}

#[test]
fn test_or() {
    let (regs, _) = run_and_regs("1.0.0.12\n1.0.1.10\n3.1.0.1\n0.1.0.0\n");
    assert_eq!(regs[0], 12 | 10);
}

#[test]
fn test_shl() {
    let (regs, _) = run_and_regs("1.0.0.1\n3.4.0.3\n0.1.0.0\n");
    assert_eq!(regs[0], 8);
}

#[test]
fn test_shr() {
    let (regs, _) = run_and_regs("1.0.0.16\n3.5.0.2\n0.1.0.0\n");
    assert_eq!(regs[0], 4);
}

// ── Category 4 ──────────────────────────────────────────────────────────────

#[test]
fn test_eq_true() {
    let (_, flags) = run_and_regs("1.0.0.5\n1.0.1.5\n4.1.0.1\n0.1.0.0\n");
    assert!(flags);
}

#[test]
fn test_eq_false() {
    let (_, flags) = run_and_regs("1.0.0.5\n1.0.1.6\n4.1.0.1\n0.1.0.0\n");
    assert!(!flags);
}

#[test]
fn test_lt() {
    let (_, flags) = run_and_regs("1.0.0.3\n1.0.1.5\n4.3.0.1\n0.1.0.0\n");
    assert!(flags);
}

#[test]
fn test_gt() {
    let (_, flags) = run_and_regs("1.0.0.10\n1.0.1.5\n4.4.0.1\n0.1.0.0\n");
    assert!(flags);
}

#[test]
fn test_gte_equal() {
    let (_, flags) = run_and_regs("1.0.0.5\n1.0.1.5\n4.6.0.1\n0.1.0.0\n");
    assert!(flags);
}

// ── Category 5 ──────────────────────────────────────────────────────────────

#[test]
fn test_jmp() {
    // JMP to line 3 (skip line 2)
    // line 1: LOAD R0, 1
    // line 2: JMP 4
    // line 3: LOAD R0, 99  ← skipped
    // line 4: HALT
    let src = "1.0.0.1\n5.0.0.4\n1.0.0.99\n0.1.0.0\n";
    let (regs, _) = run_and_regs(src);
    assert_eq!(regs[0], 1);
}

#[test]
fn test_jt_taken() {
    // FLAGS=true → JT 4 skips line 3
    let src = "1.0.0.5\n4.1.0.0\n5.1.0.5\n1.0.0.99\n0.1.0.0\n";
    let (regs, _) = run_and_regs(src);
    assert_eq!(regs[0], 5);
}

#[test]
fn test_jf_taken() {
    // FLAGS=false → JF 4 skips line 3
    let src = "1.0.0.5\n1.0.1.9\n4.1.0.1\n5.2.0.6\n1.0.0.99\n0.1.0.0\n";
    let (regs, _) = run_and_regs(src);
    assert_eq!(regs[0], 5);
}

#[test]
fn test_invalid_jump() {
    let tokens = lex("5.0.0.99\n", "t.ipc").unwrap();
    let program = parse(&tokens, "t.ipc").unwrap();
    let mut vm = Vm::new("t.ipc");
    let err = vm.run_program(&program).unwrap_err();
    assert_eq!(err.kind, ErrorKind::InvalidJump);
}

// ── Category 7 ──────────────────────────────────────────────────────────────

#[test]
fn test_call_ret() {
    // line 1: JMP 4         (skip function body)
    // line 2: INC R0        (function: increment R0)
    // line 3: RET
    // line 4: LOAD R0, 10
    // line 5: CALL 2
    // line 6: HALT
    let src = "5.0.0.4\n2.5.0.0\n7.1.0.0\n1.0.0.10\n7.0.0.2\n0.1.0.0\n";
    let (regs, _) = run_and_regs(src);
    assert_eq!(regs[0], 11);
}

#[test]
fn test_frame_unframe_preserve_existing_stack_values() {
    let src = "1.0.0.7\n1.4.0.0\n7.3.2.0\n7.4.2.0\n1.5.1.0\n0.1.0.0\n";
    let (regs, _) = run_and_regs(src);
    assert_eq!(regs[1], 7);
}

#[test]
fn test_unframe_underflow() {
    let tokens = lex("7.4.1.0\n", "t.ipc").unwrap();
    let program = parse(&tokens, "t.ipc").unwrap();
    let mut vm = Vm::new("t.ipc");
    let err = vm.run_program(&program).unwrap_err();
    assert_eq!(err.kind, ErrorKind::StackUnderflow);
}

// ── Cycle limit ──────────────────────────────────────────────────────────────

#[test]
fn test_cycle_limit() {
    let tokens = lex("5.0.0.1\n", "t.ipc").unwrap();
    let program = parse(&tokens, "t.ipc").unwrap();
    let mut vm = Vm::new("t.ipc").with_cycle_limit(100);
    let err = vm.run_program(&program).unwrap_err();
    assert_eq!(err.kind, ErrorKind::CycleLimitExceeded);
}
