# IPcode 🌐

**IPcode** is a programming language where every single line of source code is written as a fake IPv4 address.

```
1.0.0.72     # LOAD R0, 72  ('H')
6.1.0.0      # PRINTC R0
1.0.0.105    # LOAD R0, 105 ('i')
6.1.0.0      # PRINTC R0
6.3.0.0      # PRINTLN
0.1.0.0      # HALT
```

No keywords. No symbols. Just dots and numbers — and the illusion of a subnet.

---

## Table of Contents

- [Core Concept](#core-concept)
- [Installation](#installation)
- [Usage](#usage)
- [Instruction Set Reference](#instruction-set-reference)
- [Multi-Byte Value Encoding](#multi-byte-value-encoding)
- [Sample Programs](#sample-programs)
- [VM Specification](#vm-specification)

---

## Core Concept

Every IPcode source file (`.ipc`) consists of lines in the form:

```
A.B.C.D
```

Each octet encodes a different part of the instruction:

| Octet | Role                                        |
|-------|---------------------------------------------|
| `A`   | Opcode category (0–10)                      |
| `B`   | Sub-opcode / primary operand                |
| `C`   | Second operand (register index, address…)   |
| `D`   | Modifier / third operand (flags, offsets…)  |

Lines starting with `#` are comments and are ignored by the interpreter.

---

## Installation

### Pre-built installers

Download the latest release for your platform from the [Releases page](../../releases).

#### Linux packages

- **Debian/Ubuntu** (`.deb`):
  ```sh
  sudo apt install ./ipcode-*.deb
  ```
- **Fedora/RHEL** (`.rpm`):
  ```sh
  sudo dnf install ./ipcode-*.rpm
  ```

### Build from source

You need Rust 1.70+ and Cargo:

```sh
git clone https://github.com/m4rcel-lol/ipcode.git
cd ipcode
cargo build --release
# Binary is at target/release/ipcode (or ipcode.exe on Windows)
```

---

## Usage

```
ipcode run <file.ipc>       Execute an IPcode program
ipcode check <file.ipc>     Validate syntax without running
ipcode disasm <file.ipc>    Print human-readable disassembly
ipcode debug <file.ipc>     Step-through debug mode (prints register state)
ipcode version              Print the IPcode version
ipcode help                 Show usage information
```

### Options

```
--cycle-limit <N>    Override default cycle limit (default: 10,000,000)
```

### Examples

```sh
ipcode run tests/programs/hello.ipc
ipcode disasm tests/programs/add.ipc
ipcode debug tests/programs/func.ipc
```

---

## Instruction Set Reference

### Category 0 — System Control

| Address   | Instruction  | Description            |
|-----------|--------------|------------------------|
| `0.0.0.0` | `NOP`        | No operation           |
| `0.1.0.0` | `HALT`       | Stop execution         |
| `0.2.B.0` | `SYSCALL B`  | Call system function B |

### Category 1 — Data / Register Operations

| Address    | Instruction    | Description                                    |
|------------|----------------|------------------------------------------------|
| `1.0.R.V`  | `LOAD R, V`    | Load literal value V into register R           |
| `1.1.R.M`  | `LOADM R, M`   | Load value from memory address M into R        |
| `1.2.R.M`  | `STORE R, M`   | Store register R into memory address M         |
| `1.3.Ra.Rb`| `MOV Ra, Rb`   | Copy register Rb into Ra                       |
| `1.4.R.0`  | `PUSH R`       | Push register R onto the data stack            |
| `1.5.R.0`  | `POP R`        | Pop top of stack into register R               |
| `1.6.R.V`  | `LOADHI R, V`  | Load V into upper byte of R (R = (V<<8)\|lo)  |

### Category 2 — Arithmetic

| Address    | Instruction  | Description  |
|------------|--------------|--------------|
| `2.0.Ra.Rb`| `ADD Ra, Rb` | Ra = Ra + Rb |
| `2.1.Ra.Rb`| `SUB Ra, Rb` | Ra = Ra - Rb |
| `2.2.Ra.Rb`| `MUL Ra, Rb` | Ra = Ra * Rb |
| `2.3.Ra.Rb`| `DIV Ra, Rb` | Ra = Ra / Rb |
| `2.4.Ra.Rb`| `MOD Ra, Rb` | Ra = Ra % Rb |
| `2.5.R.0`  | `INC R`      | R = R + 1    |
| `2.6.R.0`  | `DEC R`      | R = R - 1    |
| `2.7.Ra.Rb`| `NEG Ra, Rb` | Ra = -Rb     |

### Category 3 — Bitwise Logic

| Address    | Instruction  | Description   |
|------------|--------------|---------------|
| `3.0.Ra.Rb`| `AND Ra, Rb` | Ra = Ra & Rb  |
| `3.1.Ra.Rb`| `OR Ra, Rb`  | Ra = Ra \| Rb |
| `3.2.Ra.Rb`| `XOR Ra, Rb` | Ra = Ra ^ Rb  |
| `3.3.R.0`  | `NOT R`      | R = ~R        |
| `3.4.R.V`  | `SHL R, V`   | R = R << V    |
| `3.5.R.V`  | `SHR R, V`   | R = R >> V    |

### Category 4 — Comparison (result stored in FLAGS register)

| Address    | Instruction   | Description        |
|------------|---------------|--------------------|
| `4.0.Ra.Rb`| `CMP Ra, Rb`  | FLAGS = (Ra == Rb) |
| `4.1.Ra.Rb`| `EQ Ra, Rb`   | FLAGS = (Ra == Rb) |
| `4.2.Ra.Rb`| `NEQ Ra, Rb`  | FLAGS = (Ra != Rb) |
| `4.3.Ra.Rb`| `LT Ra, Rb`   | FLAGS = (Ra < Rb)  |
| `4.4.Ra.Rb`| `GT Ra, Rb`   | FLAGS = (Ra > Rb)  |
| `4.5.Ra.Rb`| `LTE Ra, Rb`  | FLAGS = (Ra <= Rb) |
| `4.6.Ra.Rb`| `GTE Ra, Rb`  | FLAGS = (Ra >= Rb) |

### Category 5 — Jump / Control Flow

| Address   | Instruction | Description                          |
|-----------|-------------|--------------------------------------|
| `5.0.0.L` | `JMP L`     | Unconditional jump to line L         |
| `5.1.0.L` | `JT L`      | Jump to L if FLAGS is true           |
| `5.2.0.L` | `JF L`      | Jump to L if FLAGS is false          |
| `5.3.R.L` | `JR R, L`   | Jump to L if R is not zero           |
| `5.4.R.L` | `JZ R, L`   | Jump to L if R is zero               |
| `5.5.0.L` | `LOOP L`    | Decrement R15, jump to L if R15 != 0 |

### Category 6 — I/O

| Address   | Instruction | Description                                         |
|-----------|-------------|-----------------------------------------------------|
| `6.0.R.0` | `PRINTI R`  | Print register R as integer                         |
| `6.1.R.0` | `PRINTC R`  | Print register R as ASCII character                 |
| `6.2.R.0` | `PRINTS R`  | Print null-terminated string at memory address in R |
| `6.3.0.0` | `PRINTLN`   | Print a newline                                     |
| `6.4.R.0` | `INPUTI R`  | Read integer from stdin into R                      |
| `6.5.R.0` | `INPUTC R`  | Read single char from stdin into R                  |

### Category 7 — Functions / Call Stack

| Address   | Instruction  | Description                        |
|-----------|--------------|------------------------------------|
| `7.0.0.L` | `CALL L`     | Push return address, jump to L     |
| `7.1.0.0` | `RET`        | Pop return address and jump to it  |
| `7.2.R.0` | `CALLR R`    | Call function at address in R      |
| `7.3.V.0` | `FRAME V`    | Allocate V stack frame slots       |
| `7.4.V.0` | `UNFRAME V`  | Deallocate V stack frame slots     |

### Category 8 — Memory / Heap

| Address    | Instruction     | Description                                  |
|------------|-----------------|----------------------------------------------|
| `8.0.R.V`  | `ALLOC R, V`    | Allocate V bytes on heap; store address in R |
| `8.1.R.0`  | `FREE R`        | Free heap memory at address in R             |
| `8.2.Ra.Rb`| `MREAD Ra, Rb`  | Read memory at address in Rb into Ra         |
| `8.3.Ra.Rb`| `MWRITE Ra, Rb` | Write Ra into memory at address in Rb        |

### Category 9 — String Operations

| Address    | Instruction   | Description                              |
|------------|---------------|------------------------------------------|
| `9.0.Ra.Rb`| `SLEN Ra, Rb` | Ra = length of string at address Rb      |
| `9.1.Ra.Rb`| `SCAT Ra, Rb` | Concatenate string at Rb onto string Ra  |
| `9.2.Ra.Rb`| `SCPY Ra, Rb` | Copy string at Rb into Ra                |
| `9.3.Ra.Rb`| `SCMP Ra, Rb` | Compare strings, set FLAGS               |

### Category 10 — Arrays

| Address     | Instruction    | Description                                      |
|-------------|----------------|--------------------------------------------------|
| `10.0.R.V`  | `ANEW R, V`    | Allocate array of V elements; store address in R |
| `10.1.Ra.Rb`| `AGET Ra, Rb`  | Ra = array[Rb]                                   |
| `10.2.Ra.Rb`| `ASET Ra, Rb`  | array[Ra] = Rb                                   |
| `10.3.R.0`  | `ALEN R`       | R = length of array at address R                 |

---

## Multi-Byte Value Encoding

Because each octet is limited to 0–255, values larger than 255 require two
instructions: `LOAD` to set the lower byte and `LOADHI` to set the upper byte.

```
# Load 25601 (= 100 << 8 | 1) into R0
1.0.0.1      # LOAD R0, 1       — R0 = 1
1.6.0.100    # LOADHI R0, 100   — R0 = (100 << 8) | 1 = 25601
```

The formula is:

```
LOAD R, lo_byte          R = lo_byte
LOADHI R, hi_byte        R = (hi_byte << 8) | lo_byte
```

For 16-bit maximum (0–65535):

```
# Load 65535 into R0
1.0.0.255    # R0 = 255
1.6.0.255    # R0 = (255 << 8) | 255 = 65535
```

---

## Sample Programs

### hello.ipc — Print "Hi" and halt

```
# Hello World in IPcode
1.0.0.72     # LOAD R0, 72  (ASCII 'H')
6.1.0.0      # PRINTC R0
1.0.0.105    # LOAD R0, 105 (ASCII 'i')
6.1.0.0      # PRINTC R0
6.3.0.0      # PRINTLN
0.1.0.0      # HALT
```

```sh
ipcode run tests/programs/hello.ipc
# Output: Hi
```

### add.ipc — Add two numbers and print the result

```
# Add 40 + 2 and print result
1.0.0.40     # LOAD R0, 40
1.0.1.2      # LOAD R1, 2
2.0.0.1      # ADD R0, R1
6.0.0.0      # PRINTI R0
6.3.0.0      # PRINTLN
0.1.0.0      # HALT
```

```sh
ipcode run tests/programs/add.ipc
# Output: 42
```

### loop.ipc — Count from 1 to 5

```
# Count 1 to 5
1.0.0.1      # LOAD R0, 1   (counter)
1.0.1.5      # LOAD R1, 5   (limit)
6.0.0.0      # PRINTI R0    (line 3, loop start)
6.3.0.0      # PRINTLN
2.5.0.0      # INC R0
4.6.0.1      # GTE R0, R1
5.2.0.3      # JF 3         (jump back to line 3 if not done)
0.1.0.0      # HALT
```

```sh
ipcode run tests/programs/loop.ipc
# Output: 1 2 3 4 5 (each on its own line)
```

### func.ipc — Call a function that squares a number

```
5.0.0.4      # JMP 4         (skip to main at line 4)
2.2.0.0      # MUL R0, R0   (line 2: square R0)
7.1.0.0      # RET          (line 3)
1.0.0.7      # LOAD R0, 7   (line 4)
7.0.0.2      # CALL 2       (line 5)
6.0.0.0      # PRINTI R0    (line 6)
6.3.0.0      # PRINTLN      (line 7)
0.1.0.0      # HALT         (line 8)
```

```sh
ipcode run tests/programs/func.ipc
# Output: 49
```

### calculator.ipc — Interactive calculator

Supports `+`, `-`, `*`, `/`, `%` and loops until you enter `q` as operator.

```sh
ipcode run tests/programs/calculator.ipc
```

---

## VM Specification

| Component       | Details                                           |
|-----------------|---------------------------------------------------|
| Registers       | 16 general-purpose (`R0`–`R15`), 64-bit signed   |
| FLAGS register  | Boolean result of last comparison instruction     |
| Program Counter | 0-based index; advances after each instruction    |
| Data Stack      | For `PUSH`/`POP`; depth limit 1024               |
| Call Stack      | For `CALL`/`RET`; depth limit 256                |
| Heap            | Dynamic memory via `ALLOC`/`FREE`/`MREAD`/`MWRITE`|
| Cycle limit     | Default 10,000,000; configurable via `--cycle-limit` |
| Line numbers    | 1-based; comments and blank lines do not count    |

---

## License

MIT — see [LICENSE](LICENSE).
