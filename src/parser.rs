//! Parser for IPcode — converts raw `(A, B, C, D)` [`Token`]s into typed
//! [`Instruction`] values.

use crate::errors::{ErrorKind, IpcError};
use crate::instructions::Instruction;
use crate::lexer::Token;

/// Parse a slice of [`Token`]s into a list of [`Instruction`]s.
///
/// # Errors
/// Returns an [`IpcError`] for any token whose opcode combination is not
/// recognized or whose operands are invalid (e.g. register index > 15).
pub fn parse(tokens: &[Token], file: &str) -> Result<Vec<Instruction>, IpcError> {
    tokens
        .iter()
        .map(|t| parse_token(t, file))
        .collect()
}

/// Parse a single [`Token`] into an [`Instruction`].
fn parse_token(t: &Token, file: &str) -> Result<Instruction, IpcError> {
    let raw = format!("{}.{}.{}.{}", t.a, t.b, t.c, t.d);

    macro_rules! reg {
        ($r:expr) => {{
            if $r > 15 {
                return Err(IpcError::new(
                    ErrorKind::InvalidRegister,
                    file,
                    t.source_line,
                    Some(raw.clone()),
                    format!("register index {} is out of range (0–15)", $r),
                ));
            }
            $r
        }};
    }

    let instr = match t.a {
        // ── Category 0: System Control ────────────────────────────────────
        0 => match t.b {
            0 => Instruction::Nop,
            1 => Instruction::Halt,
            2 => Instruction::Syscall(t.c),
            _ => return Err(unknown_opcode(t, file, &raw)),
        },

        // ── Category 1: Data / Register Operations ────────────────────────
        1 => match t.b {
            0 => Instruction::Load(reg!(t.c), t.d),
            1 => Instruction::Loadm(reg!(t.c), t.d),
            2 => Instruction::Store(reg!(t.c), t.d),
            3 => Instruction::Mov(reg!(t.c), reg!(t.d)),
            4 => Instruction::Push(reg!(t.c)),
            5 => Instruction::Pop(reg!(t.c)),
            6 => Instruction::Loadhi(reg!(t.c), t.d),
            _ => return Err(unknown_opcode(t, file, &raw)),
        },

        // ── Category 2: Arithmetic ─────────────────────────────────────────
        2 => match t.b {
            0 => Instruction::Add(reg!(t.c), reg!(t.d)),
            1 => Instruction::Sub(reg!(t.c), reg!(t.d)),
            2 => Instruction::Mul(reg!(t.c), reg!(t.d)),
            3 => Instruction::Div(reg!(t.c), reg!(t.d)),
            4 => Instruction::Mod(reg!(t.c), reg!(t.d)),
            5 => Instruction::Inc(reg!(t.c)),
            6 => Instruction::Dec(reg!(t.c)),
            7 => Instruction::Neg(reg!(t.c), reg!(t.d)),
            _ => return Err(unknown_opcode(t, file, &raw)),
        },

        // ── Category 3: Bitwise Logic ──────────────────────────────────────
        3 => match t.b {
            0 => Instruction::And(reg!(t.c), reg!(t.d)),
            1 => Instruction::Or(reg!(t.c), reg!(t.d)),
            2 => Instruction::Xor(reg!(t.c), reg!(t.d)),
            3 => Instruction::Not(reg!(t.c)),
            4 => Instruction::Shl(reg!(t.c), t.d),
            5 => Instruction::Shr(reg!(t.c), t.d),
            _ => return Err(unknown_opcode(t, file, &raw)),
        },

        // ── Category 4: Comparison ─────────────────────────────────────────
        4 => match t.b {
            0 => Instruction::Cmp(reg!(t.c), reg!(t.d)),
            1 => Instruction::Eq(reg!(t.c), reg!(t.d)),
            2 => Instruction::Neq(reg!(t.c), reg!(t.d)),
            3 => Instruction::Lt(reg!(t.c), reg!(t.d)),
            4 => Instruction::Gt(reg!(t.c), reg!(t.d)),
            5 => Instruction::Lte(reg!(t.c), reg!(t.d)),
            6 => Instruction::Gte(reg!(t.c), reg!(t.d)),
            _ => return Err(unknown_opcode(t, file, &raw)),
        },

        // ── Category 5: Jump / Control Flow ───────────────────────────────
        5 => match t.b {
            0 => Instruction::Jmp(t.d),
            1 => Instruction::Jt(t.d),
            2 => Instruction::Jf(t.d),
            3 => Instruction::Jr(reg!(t.c), t.d),
            4 => Instruction::Jz(reg!(t.c), t.d),
            5 => Instruction::Loop(t.d),
            _ => return Err(unknown_opcode(t, file, &raw)),
        },

        // ── Category 6: I/O ───────────────────────────────────────────────
        6 => match t.b {
            0 => Instruction::Printi(reg!(t.c)),
            1 => Instruction::Printc(reg!(t.c)),
            2 => Instruction::Prints(reg!(t.c)),
            3 => Instruction::Println,
            4 => Instruction::Inputi(reg!(t.c)),
            5 => Instruction::Inputc(reg!(t.c)),
            _ => return Err(unknown_opcode(t, file, &raw)),
        },

        // ── Category 7: Functions / Call Stack ────────────────────────────
        7 => match t.b {
            0 => Instruction::Call(t.d),
            1 => Instruction::Ret,
            2 => Instruction::Callr(reg!(t.c)),
            3 => Instruction::Frame(t.c),
            4 => Instruction::Unframe(t.c),
            _ => return Err(unknown_opcode(t, file, &raw)),
        },

        // ── Category 8: Memory / Heap ──────────────────────────────────────
        8 => match t.b {
            0 => Instruction::Alloc(reg!(t.c), t.d),
            1 => Instruction::Free(reg!(t.c)),
            2 => Instruction::Mread(reg!(t.c), reg!(t.d)),
            3 => Instruction::Mwrite(reg!(t.c), reg!(t.d)),
            _ => return Err(unknown_opcode(t, file, &raw)),
        },

        // ── Category 9: String Operations ─────────────────────────────────
        9 => match t.b {
            0 => Instruction::Slen(reg!(t.c), reg!(t.d)),
            1 => Instruction::Scat(reg!(t.c), reg!(t.d)),
            2 => Instruction::Scpy(reg!(t.c), reg!(t.d)),
            3 => Instruction::Scmp(reg!(t.c), reg!(t.d)),
            _ => return Err(unknown_opcode(t, file, &raw)),
        },

        // ── Category 10: Arrays ────────────────────────────────────────────
        10 => match t.b {
            0 => Instruction::Anew(reg!(t.c), t.d),
            1 => Instruction::Aget(reg!(t.c), reg!(t.d)),
            2 => Instruction::Aset(reg!(t.c), reg!(t.d)),
            3 => Instruction::Alen(reg!(t.c)),
            _ => return Err(unknown_opcode(t, file, &raw)),
        },

        _ => return Err(unknown_opcode(t, file, &raw)),
    };

    Ok(instr)
}

fn unknown_opcode(t: &Token, file: &str, raw: &str) -> IpcError {
    IpcError::new(
        ErrorKind::UnknownOpcode,
        file,
        t.source_line,
        Some(raw.to_string()),
        format!("unknown opcode category {} sub-opcode {}", t.a, t.b),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    fn parse_src(src: &str) -> Result<Vec<Instruction>, IpcError> {
        let tokens = lex(src, "test.ipc").unwrap();
        parse(&tokens, "test.ipc")
    }

    #[test]
    fn test_parse_nop() {
        let instrs = parse_src("0.0.0.0\n").unwrap();
        assert_eq!(instrs[0], Instruction::Nop);
    }

    #[test]
    fn test_parse_halt() {
        let instrs = parse_src("0.1.0.0\n").unwrap();
        assert_eq!(instrs[0], Instruction::Halt);
    }

    #[test]
    fn test_parse_load() {
        let instrs = parse_src("1.0.3.99\n").unwrap();
        assert_eq!(instrs[0], Instruction::Load(3, 99));
    }

    #[test]
    fn test_parse_add() {
        let instrs = parse_src("2.0.0.1\n").unwrap();
        assert_eq!(instrs[0], Instruction::Add(0, 1));
    }

    #[test]
    fn test_parse_jmp() {
        let instrs = parse_src("5.0.0.10\n").unwrap();
        assert_eq!(instrs[0], Instruction::Jmp(10));
    }

    #[test]
    fn test_parse_println() {
        let instrs = parse_src("6.3.0.0\n").unwrap();
        assert_eq!(instrs[0], Instruction::Println);
    }

    #[test]
    fn test_parse_call_ret() {
        let instrs = parse_src("7.0.0.5\n7.1.0.0\n").unwrap();
        assert_eq!(instrs[0], Instruction::Call(5));
        assert_eq!(instrs[1], Instruction::Ret);
    }

    #[test]
    fn test_invalid_register() {
        let tokens = lex("1.0.16.0\n", "test.ipc").unwrap();
        let err = parse(&tokens, "test.ipc").unwrap_err();
        assert_eq!(err.kind, ErrorKind::InvalidRegister);
    }

    #[test]
    fn test_unknown_opcode() {
        let tokens = lex("200.0.0.0\n", "test.ipc").unwrap();
        let err = parse(&tokens, "test.ipc").unwrap_err();
        assert_eq!(err.kind, ErrorKind::UnknownOpcode);
    }

    #[test]
    fn test_parse_mov() {
        let instrs = parse_src("1.3.0.1\n").unwrap();
        assert_eq!(instrs[0], Instruction::Mov(0, 1));
    }

    #[test]
    fn test_parse_printi() {
        let instrs = parse_src("6.0.2.0\n").unwrap();
        assert_eq!(instrs[0], Instruction::Printi(2));
    }
}
