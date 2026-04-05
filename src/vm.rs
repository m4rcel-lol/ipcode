//! Register-based virtual machine for IPcode.
//!
//! The VM implements a classic fetch → decode → execute cycle over a list of
//! [`Instruction`]s.  It owns all runtime state: registers, FLAGS, the program
//! counter, both stacks, and the heap.

use crate::errors::{ErrorKind, IpcError};
use crate::instructions::Instruction;
use crate::memory::{CallStack, DataStack, HeapMemory};
use crate::io as ipc_io;

/// Number of general-purpose registers.
pub const NUM_REGISTERS: usize = 16;

/// Default maximum number of instructions the VM will execute before stopping.
pub const DEFAULT_CYCLE_LIMIT: usize = 10_000_000;

/// The state of the virtual machine after execution finishes.
#[derive(Debug)]
pub struct VmState {
    /// All 16 general-purpose registers.
    pub registers: [i64; NUM_REGISTERS],
    /// Boolean FLAGS register — result of the last comparison.
    pub flags: bool,
    /// Program Counter — index of the next instruction to execute (0-based).
    pub pc: usize,
    /// Number of instructions executed.
    pub cycles: usize,
}

/// The IPcode register-based virtual machine.
pub struct Vm {
    /// General-purpose registers R0–R15.
    registers: [i64; NUM_REGISTERS],
    /// FLAGS register.
    flags: bool,
    /// Program counter (0-based instruction index).
    pc: usize,
    /// Data stack.
    data_stack: DataStack,
    /// Call stack.
    call_stack: CallStack,
    /// Heap memory.
    heap: HeapMemory,
    /// Source file name (used in error messages).
    file: String,
    /// Maximum number of instructions to execute.
    cycle_limit: usize,
}

impl Vm {
    /// Create a new VM for the given source file.
    pub fn new(file: impl Into<String>) -> Self {
        Vm {
            registers: [0i64; NUM_REGISTERS],
            flags: false,
            pc: 0,
            data_stack: DataStack::new(1024),
            call_stack: CallStack::new(256),
            heap: HeapMemory::new(),
            file: file.into(),
            cycle_limit: DEFAULT_CYCLE_LIMIT,
        }
    }

    /// Override the default cycle limit.
    pub fn with_cycle_limit(mut self, limit: usize) -> Self {
        self.cycle_limit = limit;
        self
    }

    /// Run the program to completion.
    ///
    /// Returns a snapshot of the VM state when the program finishes (either via
    /// [`Instruction::Halt`] or by falling off the end of the program).
    ///
    /// # Errors
    /// Returns an [`IpcError`] if any runtime error occurs.
    pub fn run_program(&mut self, program: &[Instruction]) -> Result<VmState, IpcError> {
        let program_len = program.len();
        let mut cycles = 0usize;

        loop {
            if self.pc >= program_len {
                break;
            }
            if cycles >= self.cycle_limit {
                return Err(IpcError::new(
                    ErrorKind::CycleLimitExceeded,
                    &self.file,
                    self.pc + 1,
                    None,
                    format!("cycle limit of {} exceeded", self.cycle_limit),
                ));
            }
            let instr = program[self.pc].clone();
            let src_line = self.pc + 1;
            let halted = self.execute(&instr, src_line, program_len)?;
            cycles += 1;
            if halted {
                break;
            }
        }

        Ok(VmState {
            registers: self.registers,
            flags: self.flags,
            pc: self.pc,
            cycles,
        })
    }

    /// Run in debug mode — prints register state after every instruction.
    ///
    /// # Errors
    /// Returns an [`IpcError`] if any runtime error occurs.
    pub fn run_program_debug(&mut self, program: &[Instruction]) -> Result<VmState, IpcError> {
        use colored::Colorize;
        let program_len = program.len();
        let mut cycles = 0usize;

        loop {
            if self.pc >= program_len {
                break;
            }
            if cycles >= self.cycle_limit {
                return Err(IpcError::new(
                    ErrorKind::CycleLimitExceeded,
                    &self.file,
                    self.pc + 1,
                    None,
                    format!("cycle limit of {} exceeded", self.cycle_limit),
                ));
            }
            let instr = program[self.pc].clone();
            let src_line = self.pc + 1;

            println!(
                "{} {} {}",
                format!("[{:>4}]", src_line).cyan(),
                instr.mnemonic().yellow().bold(),
                instr.operands().white()
            );

            let halted = self.execute(&instr, src_line, program_len)?;
            cycles += 1;
            self.print_registers();
            if halted {
                break;
            }
        }

        Ok(VmState {
            registers: self.registers,
            flags: self.flags,
            pc: self.pc,
            cycles,
        })
    }

    /// Print all register values and FLAGS to stdout.
    fn print_registers(&self) {
        use colored::Colorize;
        print!("  FLAGS={:<6}", if self.flags { "true" } else { "false" });
        for (i, &v) in self.registers.iter().enumerate() {
            print!("  R{:<2}={:<6}", i, v);
            if (i + 1) % 8 == 0 {
                println!();
                print!("  ");
            }
        }
        println!("{}", "─".repeat(60).dimmed());
    }

    /// Execute a single instruction.
    ///
    /// Returns `true` if execution should stop (HALT).
    fn execute(
        &mut self,
        instr: &Instruction,
        src_line: usize,
        program_len: usize,
    ) -> Result<bool, IpcError> {
        let mut next_pc = self.pc + 1;

        match instr {
            // ── Category 0: System Control ────────────────────────────────
            Instruction::Nop => {}
            Instruction::Halt => return Ok(true),
            Instruction::Syscall(b) => {
                self.syscall(*b)?;
            }

            // ── Category 1: Data / Register Operations ────────────────────
            Instruction::Load(r, v) => {
                self.registers[*r as usize] = *v as i64;
            }
            Instruction::Loadm(r, m) => {
                let addr = *m as usize;
                let val = self.heap.read(addr, 0, &self.file, src_line)?;
                self.registers[*r as usize] = val;
            }
            Instruction::Store(r, m) => {
                let val = self.registers[*r as usize];
                let addr = *m as usize;
                self.heap.write(addr, 0, val, &self.file, src_line)?;
            }
            Instruction::Mov(ra, rb) => {
                self.registers[*ra as usize] = self.registers[*rb as usize];
            }
            Instruction::Push(r) => {
                let val = self.registers[*r as usize];
                self.data_stack.push(val, &self.file, src_line)?;
            }
            Instruction::Pop(r) => {
                let val = self.data_stack.pop(&self.file, src_line)?;
                self.registers[*r as usize] = val;
            }
            Instruction::Loadhi(r, v) => {
                let lo = self.registers[*r as usize] & 0xFF;
                self.registers[*r as usize] = ((*v as i64) << 8) | lo;
            }

            // ── Category 2: Arithmetic ─────────────────────────────────────
            Instruction::Add(ra, rb) => {
                self.registers[*ra as usize] =
                    self.registers[*ra as usize].wrapping_add(self.registers[*rb as usize]);
            }
            Instruction::Sub(ra, rb) => {
                self.registers[*ra as usize] =
                    self.registers[*ra as usize].wrapping_sub(self.registers[*rb as usize]);
            }
            Instruction::Mul(ra, rb) => {
                self.registers[*ra as usize] =
                    self.registers[*ra as usize].wrapping_mul(self.registers[*rb as usize]);
            }
            Instruction::Div(ra, rb) => {
                let divisor = self.registers[*rb as usize];
                if divisor == 0 {
                    return Err(IpcError::new(
                        ErrorKind::DivisionByZero,
                        &self.file,
                        src_line,
                        None,
                        "division by zero",
                    ));
                }
                self.registers[*ra as usize] /= divisor;
            }
            Instruction::Mod(ra, rb) => {
                let divisor = self.registers[*rb as usize];
                if divisor == 0 {
                    return Err(IpcError::new(
                        ErrorKind::DivisionByZero,
                        &self.file,
                        src_line,
                        None,
                        "modulo by zero",
                    ));
                }
                self.registers[*ra as usize] %= divisor;
            }
            Instruction::Inc(r) => {
                self.registers[*r as usize] = self.registers[*r as usize].wrapping_add(1);
            }
            Instruction::Dec(r) => {
                self.registers[*r as usize] = self.registers[*r as usize].wrapping_sub(1);
            }
            Instruction::Neg(ra, rb) => {
                self.registers[*ra as usize] = -self.registers[*rb as usize];
            }

            // ── Category 3: Bitwise Logic ──────────────────────────────────
            Instruction::And(ra, rb) => {
                self.registers[*ra as usize] &= self.registers[*rb as usize];
            }
            Instruction::Or(ra, rb) => {
                self.registers[*ra as usize] |= self.registers[*rb as usize];
            }
            Instruction::Xor(ra, rb) => {
                self.registers[*ra as usize] ^= self.registers[*rb as usize];
            }
            Instruction::Not(r) => {
                self.registers[*r as usize] = !self.registers[*r as usize];
            }
            Instruction::Shl(r, v) => {
                self.registers[*r as usize] <<= *v as i64;
            }
            Instruction::Shr(r, v) => {
                self.registers[*r as usize] >>= *v as i64;
            }

            // ── Category 4: Comparison ─────────────────────────────────────
            Instruction::Cmp(ra, rb) => {
                let a = self.registers[*ra as usize];
                let b = self.registers[*rb as usize];
                self.flags = a == b;
            }
            Instruction::Eq(ra, rb) => {
                self.flags = self.registers[*ra as usize] == self.registers[*rb as usize];
            }
            Instruction::Neq(ra, rb) => {
                self.flags = self.registers[*ra as usize] != self.registers[*rb as usize];
            }
            Instruction::Lt(ra, rb) => {
                self.flags = self.registers[*ra as usize] < self.registers[*rb as usize];
            }
            Instruction::Gt(ra, rb) => {
                self.flags = self.registers[*ra as usize] > self.registers[*rb as usize];
            }
            Instruction::Lte(ra, rb) => {
                self.flags = self.registers[*ra as usize] <= self.registers[*rb as usize];
            }
            Instruction::Gte(ra, rb) => {
                self.flags = self.registers[*ra as usize] >= self.registers[*rb as usize];
            }

            // ── Category 5: Jump / Control Flow ───────────────────────────
            Instruction::Jmp(l) => {
                next_pc = self.resolve_jump(*l as usize, src_line, program_len)?;
            }
            Instruction::Jt(l) => {
                if self.flags {
                    next_pc = self.resolve_jump(*l as usize, src_line, program_len)?;
                }
            }
            Instruction::Jf(l) => {
                if !self.flags {
                    next_pc = self.resolve_jump(*l as usize, src_line, program_len)?;
                }
            }
            Instruction::Jr(r, l) => {
                if self.registers[*r as usize] != 0 {
                    next_pc = self.resolve_jump(*l as usize, src_line, program_len)?;
                }
            }
            Instruction::Jz(r, l) => {
                if self.registers[*r as usize] == 0 {
                    next_pc = self.resolve_jump(*l as usize, src_line, program_len)?;
                }
            }
            Instruction::Loop(l) => {
                self.registers[15] = self.registers[15].wrapping_sub(1);
                if self.registers[15] != 0 {
                    next_pc = self.resolve_jump(*l as usize, src_line, program_len)?;
                }
            }

            // ── Category 6: I/O ───────────────────────────────────────────
            Instruction::Printi(r) => {
                ipc_io::print_integer(self.registers[*r as usize]);
            }
            Instruction::Printc(r) => {
                ipc_io::print_char(self.registers[*r as usize]);
            }
            Instruction::Prints(r) => {
                let addr = self.registers[*r as usize] as usize;
                self.print_string(addr, src_line)?;
            }
            Instruction::Println => {
                ipc_io::print_newline();
            }
            Instruction::Inputi(r) => {
                let val = ipc_io::read_integer(&self.file, src_line)?;
                self.registers[*r as usize] = val;
            }
            Instruction::Inputc(r) => {
                let val = ipc_io::read_char(&self.file, src_line)?;
                self.registers[*r as usize] = val;
            }

            // ── Category 7: Functions / Call Stack ────────────────────────
            Instruction::Call(l) => {
                self.call_stack.push(self.pc + 1, &self.file, src_line)?;
                next_pc = self.resolve_jump(*l as usize, src_line, program_len)?;
            }
            Instruction::Ret => {
                next_pc = self.call_stack.pop(&self.file, src_line)?;
            }
            Instruction::Callr(r) => {
                let target = self.registers[*r as usize] as usize;
                self.call_stack.push(self.pc + 1, &self.file, src_line)?;
                next_pc = self.resolve_jump(target, src_line, program_len)?;
            }
            Instruction::Frame(_v) => {
                // Frame allocation is a no-op in this simplified VM implementation.
            }
            Instruction::Unframe(_v) => {
                // Unframe is a no-op in this simplified VM implementation.
            }

            // ── Category 8: Memory / Heap ──────────────────────────────────
            Instruction::Alloc(r, v) => {
                let addr = self.heap.alloc(*v as usize, &self.file, src_line)?;
                self.registers[*r as usize] = addr as i64;
            }
            Instruction::Free(r) => {
                let addr = self.registers[*r as usize] as usize;
                self.heap.free(addr, &self.file, src_line)?;
            }
            Instruction::Mread(ra, rb) => {
                let addr = self.registers[*rb as usize] as usize;
                let val = self.heap.read(addr, 0, &self.file, src_line)?;
                self.registers[*ra as usize] = val;
            }
            Instruction::Mwrite(ra, rb) => {
                let val = self.registers[*ra as usize];
                let addr = self.registers[*rb as usize] as usize;
                self.heap.write(addr, 0, val, &self.file, src_line)?;
            }

            // ── Category 9: String Operations ─────────────────────────────
            Instruction::Slen(ra, rb) => {
                let addr = self.registers[*rb as usize] as usize;
                let len = self.string_len(addr, src_line)?;
                self.registers[*ra as usize] = len as i64;
            }
            Instruction::Scat(ra, rb) => {
                let src_addr = self.registers[*rb as usize] as usize;
                let dst_addr = self.registers[*ra as usize] as usize;
                self.string_concat(dst_addr, src_addr, src_line)?;
            }
            Instruction::Scpy(ra, rb) => {
                let src_addr = self.registers[*rb as usize] as usize;
                let dst_addr = self.registers[*ra as usize] as usize;
                self.string_copy(dst_addr, src_addr, src_line)?;
            }
            Instruction::Scmp(ra, rb) => {
                let a_addr = self.registers[*ra as usize] as usize;
                let b_addr = self.registers[*rb as usize] as usize;
                let cmp = self.string_compare(a_addr, b_addr, src_line)?;
                self.flags = cmp == 0;
            }

            // ── Category 10: Arrays ────────────────────────────────────────
            Instruction::Anew(r, v) => {
                // Allocate size + 1 to store array length in slot 0.
                let size = *v as usize + 1;
                let addr = self.heap.alloc(size, &self.file, src_line)?;
                self.heap.write(addr, 0, *v as i64, &self.file, src_line)?;
                self.registers[*r as usize] = addr as i64;
            }
            Instruction::Aget(ra, rb) => {
                let base = self.registers[*ra as usize] as usize;
                let idx = self.registers[*rb as usize] as usize;
                let val = self.heap.read(base, idx + 1, &self.file, src_line)?;
                self.registers[*ra as usize] = val;
            }
            Instruction::Aset(ra, rb) => {
                let base = self.registers[*ra as usize] as usize;
                let val = self.registers[*rb as usize];
                self.heap.write(base, 1, val, &self.file, src_line)?;
            }
            Instruction::Alen(r) => {
                let addr = self.registers[*r as usize] as usize;
                let len = self.heap.read(addr, 0, &self.file, src_line)?;
                self.registers[*r as usize] = len;
            }
        }

        self.pc = next_pc;
        Ok(false)
    }

    /// Resolve a 1-based jump target to a 0-based PC value.
    fn resolve_jump(
        &self,
        target_1based: usize,
        src_line: usize,
        program_len: usize,
    ) -> Result<usize, IpcError> {
        if target_1based == 0 || target_1based > program_len {
            return Err(IpcError::new(
                ErrorKind::InvalidJump,
                &self.file,
                src_line,
                None,
                format!(
                    "jump to line {} is out of range (program has {} instructions)",
                    target_1based, program_len
                ),
            ));
        }
        Ok(target_1based - 1)
    }

    /// Print a null-terminated string stored at heap address.
    fn print_string(&self, addr: usize, src_line: usize) -> Result<(), IpcError> {
        let mut offset = 0usize;
        loop {
            let val = self.heap.read(addr, offset, &self.file, src_line)?;
            if val == 0 {
                break;
            }
            ipc_io::print_char(val);
            offset += 1;
        }
        Ok(())
    }

    /// Return the length of a null-terminated string at heap address.
    fn string_len(&self, addr: usize, src_line: usize) -> Result<usize, IpcError> {
        let mut len = 0usize;
        loop {
            let val = self.heap.read(addr, len, &self.file, src_line)?;
            if val == 0 {
                break;
            }
            len += 1;
        }
        Ok(len)
    }

    /// Copy the string at `src_addr` into `dst_addr`.
    fn string_copy(
        &mut self,
        dst_addr: usize,
        src_addr: usize,
        src_line: usize,
    ) -> Result<(), IpcError> {
        let mut offset = 0usize;
        loop {
            let val = self.heap.read(src_addr, offset, &self.file, src_line)?;
            self.heap.write(dst_addr, offset, val, &self.file, src_line)?;
            if val == 0 {
                break;
            }
            offset += 1;
        }
        Ok(())
    }

    /// Concatenate the string at `src_addr` onto `dst_addr`.
    fn string_concat(
        &mut self,
        dst_addr: usize,
        src_addr: usize,
        src_line: usize,
    ) -> Result<(), IpcError> {
        let dst_len = self.string_len(dst_addr, src_line)?;
        let mut src_offset = 0usize;
        loop {
            let val = self.heap.read(src_addr, src_offset, &self.file, src_line)?;
            self.heap
                .write(dst_addr, dst_len + src_offset, val, &self.file, src_line)?;
            if val == 0 {
                break;
            }
            src_offset += 1;
        }
        Ok(())
    }

    /// Compare two null-terminated strings. Returns 0 if equal, non-zero otherwise.
    fn string_compare(
        &self,
        a_addr: usize,
        b_addr: usize,
        src_line: usize,
    ) -> Result<i64, IpcError> {
        let mut offset = 0usize;
        loop {
            let a = self.heap.read(a_addr, offset, &self.file, src_line)?;
            let b = self.heap.read(b_addr, offset, &self.file, src_line)?;
            if a != b {
                return Ok(a - b);
            }
            if a == 0 {
                return Ok(0);
            }
            offset += 1;
        }
    }

    /// Expose the current register values (used by tests).
    pub fn registers(&self) -> &[i64; NUM_REGISTERS] {
        &self.registers
    }

    /// Expose the FLAGS register (used by tests).
    pub fn flags(&self) -> bool {
        self.flags
    }

    /// Handle a SYSCALL instruction.
    fn syscall(&mut self, b: u8) -> Result<(), IpcError> {
        match b {
            0 => ipc_io::print_integer(self.registers[0]),
            1 => ipc_io::print_char(self.registers[0]),
            2 => ipc_io::print_newline(),
            _ => {} // Unknown syscalls silently ignored.
        }
        Ok(())
    }
}
