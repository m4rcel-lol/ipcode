//! Heap and stack memory manager for the IPcode VM.

use crate::errors::{ErrorKind, IpcError};

/// Maximum number of simultaneously live heap allocations.
const MAX_HEAP_BLOCKS: usize = 1024;

/// A single heap allocation block.
#[derive(Debug, Clone)]
struct HeapBlock {
    /// The unique address key (block index + 1, so zero is never a valid address).
    address: usize,
    /// The allocated bytes.
    data: Vec<i64>,
}

/// Heap memory manager.
///
/// Heap addresses are opaque `usize` keys returned by [`HeapMemory::alloc`].
#[derive(Debug, Default)]
pub struct HeapMemory {
    blocks: Vec<HeapBlock>,
    next_address: usize,
}

impl HeapMemory {
    /// Create a new, empty heap.
    pub fn new() -> Self {
        HeapMemory {
            blocks: Vec::new(),
            next_address: 1,
        }
    }

    /// Allocate `size` i64 slots on the heap and return the base address.
    ///
    /// # Errors
    /// Returns a [`MemoryViolation`](ErrorKind::MemoryViolation) if the heap
    /// has too many live allocations.
    pub fn alloc(&mut self, size: usize, file: &str, line: usize) -> Result<usize, IpcError> {
        if self.blocks.len() >= MAX_HEAP_BLOCKS {
            return Err(IpcError::new(
                ErrorKind::MemoryViolation,
                file,
                line,
                None,
                "heap allocation limit reached",
            ));
        }
        let address = self.next_address;
        self.next_address += 1;
        self.blocks.push(HeapBlock {
            address,
            data: vec![0i64; size.max(1)],
        });
        Ok(address)
    }

    /// Free the heap block at `address`.
    ///
    /// # Errors
    /// Returns a [`MemoryViolation`](ErrorKind::MemoryViolation) if the address
    /// does not correspond to a live allocation.
    pub fn free(&mut self, address: usize, file: &str, line: usize) -> Result<(), IpcError> {
        let pos = self.blocks.iter().position(|b| b.address == address);
        match pos {
            Some(i) => {
                self.blocks.remove(i);
                Ok(())
            }
            None => Err(IpcError::new(
                ErrorKind::MemoryViolation,
                file,
                line,
                None,
                format!("attempt to free invalid heap address {}", address),
            )),
        }
    }

    /// Read the value at `address + offset`.
    ///
    /// # Errors
    /// Returns a [`MemoryViolation`](ErrorKind::MemoryViolation) on invalid
    /// address or out-of-bounds offset.
    pub fn read(
        &self,
        address: usize,
        offset: usize,
        file: &str,
        line: usize,
    ) -> Result<i64, IpcError> {
        let block = self.find_block(address, file, line)?;
        block.data.get(offset).copied().ok_or_else(|| {
            IpcError::new(
                ErrorKind::MemoryViolation,
                file,
                line,
                None,
                format!(
                    "heap read out of bounds: address {} offset {} (size {})",
                    address,
                    offset,
                    block.data.len()
                ),
            )
        })
    }

    /// Write `value` to `address + offset`.
    ///
    /// # Errors
    /// Returns a [`MemoryViolation`](ErrorKind::MemoryViolation) on invalid
    /// address or out-of-bounds offset.
    pub fn write(
        &mut self,
        address: usize,
        offset: usize,
        value: i64,
        file: &str,
        line: usize,
    ) -> Result<(), IpcError> {
        let block = self.find_block_mut(address, file, line)?;
        if offset >= block.data.len() {
            return Err(IpcError::new(
                ErrorKind::MemoryViolation,
                file,
                line,
                None,
                format!(
                    "heap write out of bounds: address {} offset {} (size {})",
                    address,
                    offset,
                    block.data.len()
                ),
            ));
        }
        block.data[offset] = value;
        Ok(())
    }

    fn find_block(&self, address: usize, file: &str, line: usize) -> Result<&HeapBlock, IpcError> {
        self.blocks.iter().find(|b| b.address == address).ok_or_else(|| {
            IpcError::new(
                ErrorKind::MemoryViolation,
                file,
                line,
                None,
                format!("invalid heap address {}", address),
            )
        })
    }

    fn find_block_mut(
        &mut self,
        address: usize,
        file: &str,
        line: usize,
    ) -> Result<&mut HeapBlock, IpcError> {
        self.blocks
            .iter_mut()
            .find(|b| b.address == address)
            .ok_or_else(|| {
                IpcError::new(
                    ErrorKind::MemoryViolation,
                    file,
                    line,
                    None,
                    format!("invalid heap address {}", address),
                )
            })
    }
}

/// Fixed-size data stack for PUSH/POP operations.
#[derive(Debug)]
pub struct DataStack {
    data: Vec<i64>,
    max_depth: usize,
}

impl DataStack {
    /// Create a new data stack with the given maximum depth.
    pub fn new(max_depth: usize) -> Self {
        DataStack {
            data: Vec::new(),
            max_depth,
        }
    }

    /// Push a value onto the stack.
    ///
    /// # Errors
    /// Returns a [`StackOverflow`](ErrorKind::StackOverflow) if the stack is full.
    pub fn push(&mut self, value: i64, file: &str, line: usize) -> Result<(), IpcError> {
        if self.data.len() >= self.max_depth {
            return Err(IpcError::new(
                ErrorKind::StackOverflow,
                file,
                line,
                None,
                format!("data stack overflow (max depth {})", self.max_depth),
            ));
        }
        self.data.push(value);
        Ok(())
    }

    /// Pop a value from the stack.
    ///
    /// # Errors
    /// Returns a [`StackUnderflow`](ErrorKind::StackUnderflow) if the stack is empty.
    pub fn pop(&mut self, file: &str, line: usize) -> Result<i64, IpcError> {
        self.data.pop().ok_or_else(|| {
            IpcError::new(
                ErrorKind::StackUnderflow,
                file,
                line,
                None,
                "data stack underflow (pop on empty stack)",
            )
        })
    }
}

/// Call stack for CALL/RET operations (holds return addresses).
#[derive(Debug)]
pub struct CallStack {
    data: Vec<usize>,
    max_depth: usize,
}

impl CallStack {
    /// Create a new call stack with the given maximum depth.
    pub fn new(max_depth: usize) -> Self {
        CallStack {
            data: Vec::new(),
            max_depth,
        }
    }

    /// Push a return address onto the call stack.
    ///
    /// # Errors
    /// Returns a [`CallStackOverflow`](ErrorKind::CallStackOverflow) if full.
    pub fn push(&mut self, addr: usize, file: &str, line: usize) -> Result<(), IpcError> {
        if self.data.len() >= self.max_depth {
            return Err(IpcError::new(
                ErrorKind::CallStackOverflow,
                file,
                line,
                None,
                format!("call stack overflow (max depth {})", self.max_depth),
            ));
        }
        self.data.push(addr);
        Ok(())
    }

    /// Pop a return address from the call stack.
    ///
    /// # Errors
    /// Returns a [`CallStackUnderflow`](ErrorKind::CallStackUnderflow) if empty.
    pub fn pop(&mut self, file: &str, line: usize) -> Result<usize, IpcError> {
        self.data.pop().ok_or_else(|| {
            IpcError::new(
                ErrorKind::CallStackUnderflow,
                file,
                line,
                None,
                "call stack underflow (RET with empty call stack)",
            )
        })
    }
}
