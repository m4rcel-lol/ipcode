//! Full `Instruction` enum and opcode definitions for IPcode.
//!
//! Every IPcode instruction is encoded as a fake IPv4 address `A.B.C.D`.
//! This module defines the decoded, typed representation of every instruction.

/// A decoded IPcode instruction.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    // ── Category 0: System Control ─────────────────────────────────────────
    /// `0.0.0.0` — No operation.
    Nop,
    /// `0.1.0.0` — Stop execution.
    Halt,
    /// `0.2.B.0` — Call system function B.
    Syscall(u8),

    // ── Category 1: Data / Register Operations ─────────────────────────────
    /// `1.0.R.V` — Load literal value V into register R.
    Load(u8, u8),
    /// `1.1.R.M` — Load value from memory address M into R.
    Loadm(u8, u8),
    /// `1.2.R.M` — Store register R into memory address M.
    Store(u8, u8),
    /// `1.3.Ra.Rb` — Copy register Rb into Ra.
    Mov(u8, u8),
    /// `1.4.R.0` — Push register R onto the data stack.
    Push(u8),
    /// `1.5.R.0` — Pop top of stack into register R.
    Pop(u8),
    /// `1.6.R.V` — Load V into the upper byte of R (R = (V << 8) | (R & 0xFF)).
    Loadhi(u8, u8),

    // ── Category 2: Arithmetic ──────────────────────────────────────────────
    /// `2.0.Ra.Rb` — Ra = Ra + Rb.
    Add(u8, u8),
    /// `2.1.Ra.Rb` — Ra = Ra - Rb.
    Sub(u8, u8),
    /// `2.2.Ra.Rb` — Ra = Ra * Rb.
    Mul(u8, u8),
    /// `2.3.Ra.Rb` — Ra = Ra / Rb.
    Div(u8, u8),
    /// `2.4.Ra.Rb` — Ra = Ra % Rb.
    Mod(u8, u8),
    /// `2.5.R.0` — R = R + 1.
    Inc(u8),
    /// `2.6.R.0` — R = R - 1.
    Dec(u8),
    /// `2.7.Ra.Rb` — Ra = -Rb.
    Neg(u8, u8),

    // ── Category 3: Bitwise Logic ───────────────────────────────────────────
    /// `3.0.Ra.Rb` — Ra = Ra & Rb.
    And(u8, u8),
    /// `3.1.Ra.Rb` — Ra = Ra | Rb.
    Or(u8, u8),
    /// `3.2.Ra.Rb` — Ra = Ra ^ Rb.
    Xor(u8, u8),
    /// `3.3.R.0` — R = ~R.
    Not(u8),
    /// `3.4.R.V` — R = R << V.
    Shl(u8, u8),
    /// `3.5.R.V` — R = R >> V.
    Shr(u8, u8),

    // ── Category 4: Comparison ──────────────────────────────────────────────
    /// `4.0.Ra.Rb` — Compare Ra and Rb, set FLAGS (negative/zero/positive).
    Cmp(u8, u8),
    /// `4.1.Ra.Rb` — FLAGS = (Ra == Rb).
    Eq(u8, u8),
    /// `4.2.Ra.Rb` — FLAGS = (Ra != Rb).
    Neq(u8, u8),
    /// `4.3.Ra.Rb` — FLAGS = (Ra < Rb).
    Lt(u8, u8),
    /// `4.4.Ra.Rb` — FLAGS = (Ra > Rb).
    Gt(u8, u8),
    /// `4.5.Ra.Rb` — FLAGS = (Ra <= Rb).
    Lte(u8, u8),
    /// `4.6.Ra.Rb` — FLAGS = (Ra >= Rb).
    Gte(u8, u8),

    // ── Category 5: Jump / Control Flow ────────────────────────────────────
    /// `5.0.0.L` — Unconditional jump to line L (1-based).
    Jmp(u8),
    /// `5.1.0.L` — Jump to L if FLAGS is true.
    Jt(u8),
    /// `5.2.0.L` — Jump to L if FLAGS is false.
    Jf(u8),
    /// `5.3.R.L` — Jump to L if R is not zero.
    Jr(u8, u8),
    /// `5.4.R.L` — Jump to L if R is zero.
    Jz(u8, u8),
    /// `5.5.0.L` — Decrement R15, jump to L if R15 != 0.
    Loop(u8),

    // ── Category 6: I/O ─────────────────────────────────────────────────────
    /// `6.0.R.0` — Print register R as an integer.
    Printi(u8),
    /// `6.1.R.0` — Print register R as an ASCII character.
    Printc(u8),
    /// `6.2.R.0` — Print null-terminated string at memory address in R.
    Prints(u8),
    /// `6.3.0.0` — Print a newline.
    Println,
    /// `6.4.R.0` — Read integer from stdin into R.
    Inputi(u8),
    /// `6.5.R.0` — Read single char from stdin into R.
    Inputc(u8),

    // ── Category 7: Functions / Call Stack ─────────────────────────────────
    /// `7.0.0.L` — Push return address, jump to line L.
    Call(u8),
    /// `7.1.0.0` — Pop return address and jump to it.
    Ret,
    /// `7.2.R.0` — Call function at address stored in R.
    Callr(u8),
    /// `7.3.V.0` — Allocate V slots on the stack frame.
    Frame(u8),
    /// `7.4.V.0` — Deallocate V stack frame slots.
    Unframe(u8),

    // ── Category 8: Memory / Heap ───────────────────────────────────────────
    /// `8.0.R.V` — Allocate V bytes on the heap; store address in R.
    Alloc(u8, u8),
    /// `8.1.R.0` — Free heap memory at address in R.
    Free(u8),
    /// `8.2.Ra.Rb` — Read memory at address in Rb into Ra.
    Mread(u8, u8),
    /// `8.3.Ra.Rb` — Write Ra into memory at address in Rb.
    Mwrite(u8, u8),

    // ── Category 9: String Operations ──────────────────────────────────────
    /// `9.0.Ra.Rb` — Ra = length of string at address Rb.
    Slen(u8, u8),
    /// `9.1.Ra.Rb` — Concatenate string at Rb onto string at Ra.
    Scat(u8, u8),
    /// `9.2.Ra.Rb` — Copy string at Rb into Ra.
    Scpy(u8, u8),
    /// `9.3.Ra.Rb` — Compare strings at Ra and Rb, set FLAGS.
    Scmp(u8, u8),

    // ── Category 10: Arrays ─────────────────────────────────────────────────
    /// `10.0.R.V` — Allocate array of V elements; store address in R.
    Anew(u8, u8),
    /// `10.1.Ra.Rb` — Ra = array[Rb].
    Aget(u8, u8),
    /// `10.2.Ra.Rb` — array[Ra] = Rb.
    Aset(u8, u8),
    /// `10.3.R.0` — R = length of array at address R.
    Alen(u8),
}

impl Instruction {
    /// Return a short mnemonic string for this instruction.
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Instruction::Nop => "NOP",
            Instruction::Halt => "HALT",
            Instruction::Syscall(_) => "SYSCALL",
            Instruction::Load(_, _) => "LOAD",
            Instruction::Loadm(_, _) => "LOADM",
            Instruction::Store(_, _) => "STORE",
            Instruction::Mov(_, _) => "MOV",
            Instruction::Push(_) => "PUSH",
            Instruction::Pop(_) => "POP",
            Instruction::Loadhi(_, _) => "LOADHI",
            Instruction::Add(_, _) => "ADD",
            Instruction::Sub(_, _) => "SUB",
            Instruction::Mul(_, _) => "MUL",
            Instruction::Div(_, _) => "DIV",
            Instruction::Mod(_, _) => "MOD",
            Instruction::Inc(_) => "INC",
            Instruction::Dec(_) => "DEC",
            Instruction::Neg(_, _) => "NEG",
            Instruction::And(_, _) => "AND",
            Instruction::Or(_, _) => "OR",
            Instruction::Xor(_, _) => "XOR",
            Instruction::Not(_) => "NOT",
            Instruction::Shl(_, _) => "SHL",
            Instruction::Shr(_, _) => "SHR",
            Instruction::Cmp(_, _) => "CMP",
            Instruction::Eq(_, _) => "EQ",
            Instruction::Neq(_, _) => "NEQ",
            Instruction::Lt(_, _) => "LT",
            Instruction::Gt(_, _) => "GT",
            Instruction::Lte(_, _) => "LTE",
            Instruction::Gte(_, _) => "GTE",
            Instruction::Jmp(_) => "JMP",
            Instruction::Jt(_) => "JT",
            Instruction::Jf(_) => "JF",
            Instruction::Jr(_, _) => "JR",
            Instruction::Jz(_, _) => "JZ",
            Instruction::Loop(_) => "LOOP",
            Instruction::Printi(_) => "PRINTI",
            Instruction::Printc(_) => "PRINTC",
            Instruction::Prints(_) => "PRINTS",
            Instruction::Println => "PRINTLN",
            Instruction::Inputi(_) => "INPUTI",
            Instruction::Inputc(_) => "INPUTC",
            Instruction::Call(_) => "CALL",
            Instruction::Ret => "RET",
            Instruction::Callr(_) => "CALLR",
            Instruction::Frame(_) => "FRAME",
            Instruction::Unframe(_) => "UNFRAME",
            Instruction::Alloc(_, _) => "ALLOC",
            Instruction::Free(_) => "FREE",
            Instruction::Mread(_, _) => "MREAD",
            Instruction::Mwrite(_, _) => "MWRITE",
            Instruction::Slen(_, _) => "SLEN",
            Instruction::Scat(_, _) => "SCAT",
            Instruction::Scpy(_, _) => "SCPY",
            Instruction::Scmp(_, _) => "SCMP",
            Instruction::Anew(_, _) => "ANEW",
            Instruction::Aget(_, _) => "AGET",
            Instruction::Aset(_, _) => "ASET",
            Instruction::Alen(_) => "ALEN",
        }
    }

    /// Return a human-readable operand string for disassembly output.
    pub fn operands(&self) -> String {
        match self {
            Instruction::Nop | Instruction::Halt | Instruction::Ret | Instruction::Println => {
                String::new()
            }
            Instruction::Syscall(b) => format!("{}", b),
            Instruction::Load(r, v) => format!("R{}, {}", r, v),
            Instruction::Loadm(r, m) => format!("R{}, [{}]", r, m),
            Instruction::Store(r, m) => format!("R{}, [{}]", r, m),
            Instruction::Mov(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Push(r) => format!("R{}", r),
            Instruction::Pop(r) => format!("R{}", r),
            Instruction::Loadhi(r, v) => format!("R{}, {}", r, v),
            Instruction::Add(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Sub(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Mul(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Div(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Mod(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Inc(r) => format!("R{}", r),
            Instruction::Dec(r) => format!("R{}", r),
            Instruction::Neg(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::And(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Or(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Xor(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Not(r) => format!("R{}", r),
            Instruction::Shl(r, v) => format!("R{}, {}", r, v),
            Instruction::Shr(r, v) => format!("R{}, {}", r, v),
            Instruction::Cmp(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Eq(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Neq(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Lt(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Gt(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Lte(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Gte(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Jmp(l) => format!("{}", l),
            Instruction::Jt(l) => format!("{}", l),
            Instruction::Jf(l) => format!("{}", l),
            Instruction::Jr(r, l) => format!("R{}, {}", r, l),
            Instruction::Jz(r, l) => format!("R{}, {}", r, l),
            Instruction::Loop(l) => format!("{}", l),
            Instruction::Printi(r) => format!("R{}", r),
            Instruction::Printc(r) => format!("R{}", r),
            Instruction::Prints(r) => format!("R{}", r),
            Instruction::Inputi(r) => format!("R{}", r),
            Instruction::Inputc(r) => format!("R{}", r),
            Instruction::Call(l) => format!("{}", l),
            Instruction::Callr(r) => format!("R{}", r),
            Instruction::Frame(v) => format!("{}", v),
            Instruction::Unframe(v) => format!("{}", v),
            Instruction::Alloc(r, v) => format!("R{}, {}", r, v),
            Instruction::Free(r) => format!("R{}", r),
            Instruction::Mread(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Mwrite(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Slen(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Scat(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Scpy(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Scmp(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Anew(r, v) => format!("R{}, {}", r, v),
            Instruction::Aget(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Aset(ra, rb) => format!("R{}, R{}", ra, rb),
            Instruction::Alen(r) => format!("R{}", r),
        }
    }
}
