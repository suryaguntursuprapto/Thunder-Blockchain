// ---------------------------------------------------------------------------
//  Thunder Blockchain — VM Opcodes
// ---------------------------------------------------------------------------
//  Defines the bytecode instruction set for the Thunder Virtual Machine.
// ---------------------------------------------------------------------------

use serde::{Deserialize, Serialize};

/// The complete opcode set for the Thunder VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    // ── Stack Operations ───────────────────────────────────────────────
    /// Push a u64 value onto the stack.
    Push = 0x01,
    /// Pop the top value from the stack.
    Pop = 0x02,
    /// Duplicate the top value.
    Dup = 0x03,
    /// Swap the top two values.
    Swap = 0x04,

    // ── Arithmetic ─────────────────────────────────────────────────────
    /// a + b
    Add = 0x10,
    /// a - b
    Sub = 0x11,
    /// a * b
    Mul = 0x12,
    /// a / b
    Div = 0x13,
    /// a % b
    Mod = 0x14,

    // ── Comparison ─────────────────────────────────────────────────────
    /// a == b → push 1 or 0
    Eq = 0x20,
    /// a != b → push 1 or 0
    Neq = 0x21,
    /// a < b → push 1 or 0
    Lt = 0x22,
    /// a > b → push 1 or 0
    Gt = 0x23,
    /// a <= b → push 1 or 0
    Lte = 0x24,
    /// a >= b → push 1 or 0
    Gte = 0x25,

    // ── Logic ──────────────────────────────────────────────────────────
    /// Logical AND
    And = 0x30,
    /// Logical OR
    Or = 0x31,
    /// Logical NOT
    Not = 0x32,

    // ── Control Flow ───────────────────────────────────────────────────
    /// Unconditional jump to address.
    Jump = 0x40,
    /// Conditional jump (jump if top of stack is non-zero).
    JumpIf = 0x41,
    /// Call a function at address (push return address).
    Call = 0x42,
    /// Return from function call.
    Return = 0x43,
    /// Halt execution (success).
    Halt = 0x44,
    /// Revert execution (failure, with error message).
    Revert = 0x45,

    // ── Memory (Local variable array) ──────────────────────────────────
    /// Load local variable by index.
    LoadLocal = 0x50,
    /// Store value to local variable by index.
    StoreLocal = 0x51,

    // ── Memory (Linear Sandbox) ────────────────────────────────────────
    /// Load 8 bytes (u64) from linear memory at offset.
    MLoad = 0x52,
    /// Store 8 bytes (u64) into linear memory at offset.
    MStore = 0x53,

    // ── Contract Storage ───────────────────────────────────────────────
    /// Load from persistent contract storage (key → value).
    SLoad = 0x60,
    /// Store to persistent contract storage (key, value →).
    SStore = 0x61,

    // ── Blockchain Context ─────────────────────────────────────────────
    /// Push the caller's address onto the stack.
    Caller = 0x70,
    /// Push the balance of an address onto the stack.
    Balance = 0x71,
    /// Transfer coins from contract to an address.
    Transfer = 0x72,
    /// Push the current block timestamp.
    Timestamp = 0x73,
    /// Push the current block height.
    BlockHeight = 0x74,
    /// Push the contract's own address.
    SelfAddress = 0x75,

    // ── Data & Cryptography ────────────────────────────────────────────
    /// Push a byte array into linear memory, pushes offset + length.
    PushBytes = 0x80,
    /// Compute SHA-256 hash of top stack value (legacy).
    Hash = 0x81,
    /// Compute Keccak-256 Hash of a memory slice (ptr, len → hash value).
    Keccak256 = 0x82,
    /// Verify Ed25519 Signature reading from memory (msg_ptr, pubkey_ptr, sig_ptr → 1/0).
    VerifySig = 0x83,

    // ── Events ─────────────────────────────────────────────────────────
    /// Emit a log event (topic_count on stack, then topics, then data).
    Log = 0x90,

    // ── Assertions ─────────────────────────────────────────────────────
    /// Require condition (revert if top of stack is 0).
    Require = 0xA0,
}

impl OpCode {
    /// Decode a single byte into an opcode.
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::Push),
            0x02 => Some(Self::Pop),
            0x03 => Some(Self::Dup),
            0x04 => Some(Self::Swap),

            0x10 => Some(Self::Add),
            0x11 => Some(Self::Sub),
            0x12 => Some(Self::Mul),
            0x13 => Some(Self::Div),
            0x14 => Some(Self::Mod),

            0x20 => Some(Self::Eq),
            0x21 => Some(Self::Neq),
            0x22 => Some(Self::Lt),
            0x23 => Some(Self::Gt),
            0x24 => Some(Self::Lte),
            0x25 => Some(Self::Gte),

            0x30 => Some(Self::And),
            0x31 => Some(Self::Or),
            0x32 => Some(Self::Not),

            0x40 => Some(Self::Jump),
            0x41 => Some(Self::JumpIf),
            0x42 => Some(Self::Call),
            0x43 => Some(Self::Return),
            0x44 => Some(Self::Halt),
            0x45 => Some(Self::Revert),

            0x50 => Some(Self::LoadLocal),
            0x51 => Some(Self::StoreLocal),
            0x52 => Some(Self::MLoad),
            0x53 => Some(Self::MStore),

            0x60 => Some(Self::SLoad),
            0x61 => Some(Self::SStore),

            0x70 => Some(Self::Caller),
            0x71 => Some(Self::Balance),
            0x72 => Some(Self::Transfer),
            0x73 => Some(Self::Timestamp),
            0x74 => Some(Self::BlockHeight),
            0x75 => Some(Self::SelfAddress),

            0x80 => Some(Self::PushBytes),
            0x81 => Some(Self::Hash),
            0x82 => Some(Self::Keccak256),
            0x83 => Some(Self::VerifySig),

            0x90 => Some(Self::Log),

            0xA0 => Some(Self::Require),

            _ => None,
        }
    }

    /// Encode the opcode to a single byte.
    pub fn to_byte(self) -> u8 {
        self as u8
    }
}

// ── Instruction ────────────────────────────────────────────────────────────

/// A decoded instruction: opcode + optional operand.
#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: OpCode,
    /// Immediate operand (used by Push, Jump, Call, LoadLocal, StoreLocal, etc).
    pub operand: u64,
    /// Byte data operand (used by PushBytes).
    pub data: Vec<u8>,
}

impl Instruction {
    pub fn new(opcode: OpCode) -> Self {
        Self {
            opcode,
            operand: 0,
            data: Vec::new(),
        }
    }

    pub fn with_operand(opcode: OpCode, operand: u64) -> Self {
        Self {
            opcode,
            operand,
            data: Vec::new(),
        }
    }

    pub fn with_data(opcode: OpCode, data: Vec<u8>) -> Self {
        Self {
            opcode,
            operand: 0,
            data,
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_roundtrip() {
        let opcodes = [
            OpCode::Push,
            OpCode::Add,
            OpCode::Eq,
            OpCode::Jump,
            OpCode::SLoad,
            OpCode::Caller,
            OpCode::Log,
            OpCode::Require,
        ];
        for op in opcodes {
            let byte = op.to_byte();
            let decoded = OpCode::from_byte(byte).unwrap();
            assert_eq!(op, decoded);
        }
    }

    #[test]
    fn test_invalid_opcode() {
        assert!(OpCode::from_byte(0xFF).is_none());
    }
}
