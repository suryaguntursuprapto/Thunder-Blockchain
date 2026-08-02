// ---------------------------------------------------------------------------
//  Thunder Blockchain — Stack-Based Virtual Machine
// ---------------------------------------------------------------------------
//  Executes Thunder bytecode with gas metering, local variables, persistent
//  contract storage, and blockchain context access.
// ---------------------------------------------------------------------------

use std::collections::HashMap;

use thunder_core::crypto::{self, Address};

use crate::gas::GasMeter;
use crate::opcode::{Instruction, OpCode};

// ── Execution Context ──────────────────────────────────────────────────────

/// Blockchain context injected into the VM at execution time.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Address of the account that initiated the transaction.
    pub caller: Address,
    /// Address of the contract being executed.
    pub contract_address: Address,
    /// Value (coins) sent with this call.
    pub value: u64,
    /// Current block timestamp.
    pub timestamp: u64,
    /// Current block height.
    pub block_height: u64,
}

// ── Call Frame ─────────────────────────────────────────────────────────────

/// A single frame on the call stack (function invocation).
#[derive(Debug, Clone)]
struct CallFrame {
    /// Instruction pointer to return to after this frame completes.
    return_pc: usize,
    /// Base index in the local-variable store for this frame.
    locals_base: usize,
    /// Number of local variables allocated for this frame.
    locals_count: usize,
}

// ── Log Entry ──────────────────────────────────────────────────────────────

/// An emitted event/log.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub topics: Vec<u64>,
    pub data: u64,
}

// ── VM ─────────────────────────────────────────────────────────────────────

/// The Thunder Virtual Machine.
pub struct ThunderVm {
    /// The program (list of instructions).
    program: Vec<Instruction>,
    /// Instruction pointer.
    pc: usize,
    /// Operand stack.
    stack: Vec<u64>,
    /// Local variables (flat array, partitioned by call frames).
    locals: Vec<u64>,
    /// Call stack.
    call_stack: Vec<CallFrame>,
    /// Gas meter.
    gas: GasMeter,
    /// Blockchain context.
    ctx: ExecutionContext,
    /// Persistent contract storage (key → value).
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
    /// Emitted logs.
    pub logs: Vec<LogEntry>,
    /// Whether execution has halted.
    halted: bool,
    /// Whether execution reverted.
    reverted: bool,
    /// Revert reason (if any).
    pub revert_reason: Option<String>,
    /// Return value (top of stack upon Halt).
    pub return_value: Option<u64>,
    /// Balances of accounts (simplified — in production this is in WorldState).
    pub balances: HashMap<Address, u64>,
}

impl ThunderVm {
    /// Create a new VM loaded with a program.
    pub fn new(
        program: Vec<Instruction>,
        ctx: ExecutionContext,
        gas_limit: u64,
        storage: HashMap<Vec<u8>, Vec<u8>>,
    ) -> Self {
        let mut balances = HashMap::new();
        balances.insert(ctx.contract_address, 0);

        Self {
            program,
            pc: 0,
            stack: Vec::with_capacity(1024),
            locals: vec![0u64; 256],
            call_stack: Vec::new(),
            gas: GasMeter::new(gas_limit),
            ctx,
            storage,
            logs: Vec::new(),
            halted: false,
            reverted: false,
            revert_reason: None,
            return_value: None,
            balances,
        }
    }

    /// Run the VM until it halts, reverts, or runs out of gas.
    pub fn execute(&mut self) -> Result<VmResult, VmError> {
        while !self.halted && !self.reverted && self.pc < self.program.len() {
            self.step()?;
        }

        Ok(VmResult {
            gas_used: self.gas.used(),
            return_value: self.return_value,
            reverted: self.reverted,
            revert_reason: self.revert_reason.clone(),
            logs: self.logs.clone(),
            storage: self.storage.clone(),
        })
    }

    /// Execute a single instruction.
    fn step(&mut self) -> Result<(), VmError> {
        if self.pc >= self.program.len() {
            self.halted = true;
            return Ok(());
        }

        let instr = self.program[self.pc].clone();
        self.gas
            .consume_opcode(instr.opcode)
            .map_err(|e| VmError::OutOfGas(e.to_string()))?;

        match instr.opcode {
            // ── Stack ──────────────────────────────────────────────────
            OpCode::Push => {
                self.stack.push(instr.operand);
            }
            OpCode::Pop => {
                self.pop()?;
            }
            OpCode::Dup => {
                let val = *self.stack.last().ok_or(VmError::StackUnderflow)?;
                self.stack.push(val);
            }
            OpCode::Swap => {
                let len = self.stack.len();
                if len < 2 {
                    return Err(VmError::StackUnderflow);
                }
                self.stack.swap(len - 1, len - 2);
            }

            // ── Arithmetic ─────────────────────────────────────────────
            OpCode::Add => self.binary_op(|a, b| a.wrapping_add(b))?,
            OpCode::Sub => self.binary_op(|a, b| a.wrapping_sub(b))?,
            OpCode::Mul => self.binary_op(|a, b| a.wrapping_mul(b))?,
            OpCode::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                if b == 0 {
                    return Err(VmError::DivisionByZero);
                }
                self.stack.push(a / b);
            }
            OpCode::Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                if b == 0 {
                    return Err(VmError::DivisionByZero);
                }
                self.stack.push(a % b);
            }

            // ── Comparison ─────────────────────────────────────────────
            OpCode::Eq => self.binary_op(|a, b| if a == b { 1 } else { 0 })?,
            OpCode::Neq => self.binary_op(|a, b| if a != b { 1 } else { 0 })?,
            OpCode::Lt => self.binary_op(|a, b| if a < b { 1 } else { 0 })?,
            OpCode::Gt => self.binary_op(|a, b| if a > b { 1 } else { 0 })?,
            OpCode::Lte => self.binary_op(|a, b| if a <= b { 1 } else { 0 })?,
            OpCode::Gte => self.binary_op(|a, b| if a >= b { 1 } else { 0 })?,

            // ── Logic ──────────────────────────────────────────────────
            OpCode::And => self.binary_op(|a, b| if a != 0 && b != 0 { 1 } else { 0 })?,
            OpCode::Or => self.binary_op(|a, b| if a != 0 || b != 0 { 1 } else { 0 })?,
            OpCode::Not => {
                let a = self.pop()?;
                self.stack.push(if a == 0 { 1 } else { 0 });
            }

            // ── Control Flow ───────────────────────────────────────────
            OpCode::Jump => {
                self.pc = instr.operand as usize;
                return Ok(()); // Skip pc increment.
            }
            OpCode::JumpIf => {
                let cond = self.pop()?;
                if cond != 0 {
                    self.pc = instr.operand as usize;
                    return Ok(());
                }
            }
            OpCode::Call => {
                let frame = CallFrame {
                    return_pc: self.pc + 1,
                    locals_base: self.locals.len(),
                    locals_count: 16, // default local var slots per function
                };
                self.locals
                    .resize(frame.locals_base + frame.locals_count, 0);
                self.call_stack.push(frame);
                self.pc = instr.operand as usize;
                return Ok(());
            }
            OpCode::Return => {
                if let Some(frame) = self.call_stack.pop() {
                    self.locals.truncate(frame.locals_base);
                    self.pc = frame.return_pc;
                    return Ok(());
                } else {
                    self.halted = true;
                    self.return_value = self.stack.last().copied();
                    return Ok(());
                }
            }
            OpCode::Halt => {
                self.halted = true;
                self.return_value = self.stack.last().copied();
                return Ok(());
            }
            OpCode::Revert => {
                self.reverted = true;
                self.revert_reason = Some("execution reverted".to_string());
                return Ok(());
            }

            // ── Local Variables ────────────────────────────────────────
            OpCode::LoadLocal => {
                let idx = instr.operand as usize;
                let base = self.call_stack.last().map(|f| f.locals_base).unwrap_or(0);
                let val = self.locals.get(base + idx).copied().unwrap_or(0);
                self.stack.push(val);
            }
            OpCode::StoreLocal => {
                let idx = instr.operand as usize;
                let base = self.call_stack.last().map(|f| f.locals_base).unwrap_or(0);
                let val = self.pop()?;
                if base + idx >= self.locals.len() {
                    self.locals.resize(base + idx + 1, 0);
                }
                self.locals[base + idx] = val;
            }

            // ── Contract Storage ───────────────────────────────────────
            OpCode::SLoad => {
                let key = self.pop()?;
                let key_bytes = key.to_le_bytes().to_vec();
                let val = self
                    .storage
                    .get(&key_bytes)
                    .and_then(|v| {
                        if v.len() >= 8 {
                            Some(u64::from_le_bytes(v[..8].try_into().unwrap()))
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                self.stack.push(val);
            }
            OpCode::SStore => {
                let val = self.pop()?;
                let key = self.pop()?;
                let key_bytes = key.to_le_bytes().to_vec();
                let val_bytes = val.to_le_bytes().to_vec();
                self.storage.insert(key_bytes, val_bytes);
            }

            // ── Blockchain Context ─────────────────────────────────────
            OpCode::Caller => {
                // Push a u64 derived from the first 8 bytes of the caller address.
                let val = u64::from_le_bytes(self.ctx.caller[..8].try_into().unwrap());
                self.stack.push(val);
            }
            OpCode::Balance => {
                let addr_val = self.pop()?;
                let mut addr = [0u8; 20];
                addr[..8].copy_from_slice(&addr_val.to_le_bytes());
                let balance = self.balances.get(&addr).copied().unwrap_or(0);
                self.stack.push(balance);
            }
            OpCode::Transfer => {
                let amount = self.pop()?;
                let to_val = self.pop()?;
                let mut to = [0u8; 20];
                to[..8].copy_from_slice(&to_val.to_le_bytes());

                let contract_bal = self
                    .balances
                    .get(&self.ctx.contract_address)
                    .copied()
                    .unwrap_or(0);
                if contract_bal < amount {
                    return Err(VmError::InsufficientBalance);
                }

                *self.balances.entry(self.ctx.contract_address).or_insert(0) -= amount;
                *self.balances.entry(to).or_insert(0) += amount;
                self.stack.push(1); // success
            }
            OpCode::Timestamp => {
                self.stack.push(self.ctx.timestamp);
            }
            OpCode::BlockHeight => {
                self.stack.push(self.ctx.block_height);
            }
            OpCode::SelfAddress => {
                let val = u64::from_le_bytes(self.ctx.contract_address[..8].try_into().unwrap());
                self.stack.push(val);
            }

            // ── Data ───────────────────────────────────────────────────
            OpCode::PushBytes => {
                // Push the length of the byte data.
                self.stack.push(instr.data.len() as u64);
            }
            OpCode::Hash => {
                let val = self.pop()?;
                let hash = crypto::hash_sha256(&val.to_le_bytes());
                let hash_val = u64::from_le_bytes(hash[..8].try_into().unwrap());
                self.stack.push(hash_val);
            }

            // ── Events ─────────────────────────────────────────────────
            OpCode::Log => {
                let topic_count = self.pop()? as usize;
                let mut topics = Vec::with_capacity(topic_count);
                for _ in 0..topic_count {
                    topics.push(self.pop()?);
                }
                let data = self.pop()?;
                self.logs.push(LogEntry { topics, data });
            }

            // ── Assertions ─────────────────────────────────────────────
            OpCode::Require => {
                let condition = self.pop()?;
                if condition == 0 {
                    self.reverted = true;
                    self.revert_reason = Some("require failed".to_string());
                    return Ok(());
                }
            }
        }

        self.pc += 1;
        Ok(())
    }

    // ── Helpers ────────────────────────────────────────────────────────

    fn pop(&mut self) -> Result<u64, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn binary_op<F>(&mut self, f: F) -> Result<(), VmError>
    where
        F: FnOnce(u64, u64) -> u64,
    {
        let b = self.pop()?;
        let a = self.pop()?;
        self.stack.push(f(a, b));
        Ok(())
    }
}

// ── Result & Error Types ───────────────────────────────────────────────────

/// The result of VM execution.
#[derive(Debug, Clone)]
pub struct VmResult {
    pub gas_used: u64,
    pub return_value: Option<u64>,
    pub reverted: bool,
    pub revert_reason: Option<String>,
    pub logs: Vec<LogEntry>,
    pub storage: HashMap<Vec<u8>, Vec<u8>>,
}

/// VM runtime errors.
#[derive(Debug, thiserror::Error)]
pub enum VmError {
    #[error("stack underflow")]
    StackUnderflow,

    #[error("division by zero")]
    DivisionByZero,

    #[error("out of gas: {0}")]
    OutOfGas(String),

    #[error("insufficient balance for transfer")]
    InsufficientBalance,

    #[error("invalid instruction at pc={0}")]
    InvalidInstruction(usize),
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_ctx() -> ExecutionContext {
        ExecutionContext {
            caller: [1u8; 20],
            contract_address: [2u8; 20],
            value: 0,
            timestamp: 1000,
            block_height: 42,
        }
    }

    #[test]
    fn test_add() {
        let program = vec![
            Instruction::with_operand(OpCode::Push, 10),
            Instruction::with_operand(OpCode::Push, 20),
            Instruction::new(OpCode::Add),
            Instruction::new(OpCode::Halt),
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 100_000, HashMap::new());
        let result = vm.execute().unwrap();
        assert_eq!(result.return_value, Some(30));
        assert!(!result.reverted);
    }

    #[test]
    fn test_sub_mul() {
        let program = vec![
            Instruction::with_operand(OpCode::Push, 50),
            Instruction::with_operand(OpCode::Push, 30),
            Instruction::new(OpCode::Sub),
            Instruction::with_operand(OpCode::Push, 3),
            Instruction::new(OpCode::Mul),
            Instruction::new(OpCode::Halt),
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 100_000, HashMap::new());
        let result = vm.execute().unwrap();
        assert_eq!(result.return_value, Some(60)); // (50-30) * 3
    }

    #[test]
    fn test_division_by_zero() {
        let program = vec![
            Instruction::with_operand(OpCode::Push, 10),
            Instruction::with_operand(OpCode::Push, 0),
            Instruction::new(OpCode::Div),
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 100_000, HashMap::new());
        let result = vm.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_comparison() {
        let program = vec![
            Instruction::with_operand(OpCode::Push, 10),
            Instruction::with_operand(OpCode::Push, 20),
            Instruction::new(OpCode::Lt),
            Instruction::new(OpCode::Halt),
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 100_000, HashMap::new());
        let result = vm.execute().unwrap();
        assert_eq!(result.return_value, Some(1)); // 10 < 20 = true
    }

    #[test]
    fn test_conditional_jump() {
        // If 1 (true), jump to instruction 3 (Push 100), skip Push 999.
        let program = vec![
            Instruction::with_operand(OpCode::Push, 1),   // 0
            Instruction::with_operand(OpCode::JumpIf, 3), // 1
            Instruction::with_operand(OpCode::Push, 999), // 2 (skipped)
            Instruction::with_operand(OpCode::Push, 100), // 3
            Instruction::new(OpCode::Halt),               // 4
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 100_000, HashMap::new());
        let result = vm.execute().unwrap();
        assert_eq!(result.return_value, Some(100));
    }

    #[test]
    fn test_local_variables() {
        let program = vec![
            Instruction::with_operand(OpCode::Push, 42),
            Instruction::with_operand(OpCode::StoreLocal, 0),
            Instruction::with_operand(OpCode::Push, 0), // clear stack
            Instruction::new(OpCode::Pop),
            Instruction::with_operand(OpCode::LoadLocal, 0),
            Instruction::new(OpCode::Halt),
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 100_000, HashMap::new());
        let result = vm.execute().unwrap();
        assert_eq!(result.return_value, Some(42));
    }

    #[test]
    fn test_storage() {
        let program = vec![
            Instruction::with_operand(OpCode::Push, 1),   // key
            Instruction::with_operand(OpCode::Push, 999), // value
            Instruction::new(OpCode::SStore),
            Instruction::with_operand(OpCode::Push, 1), // key
            Instruction::new(OpCode::SLoad),
            Instruction::new(OpCode::Halt),
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 100_000, HashMap::new());
        let result = vm.execute().unwrap();
        assert_eq!(result.return_value, Some(999));
    }

    #[test]
    fn test_require_pass() {
        let program = vec![
            Instruction::with_operand(OpCode::Push, 1), // true
            Instruction::new(OpCode::Require),
            Instruction::with_operand(OpCode::Push, 42),
            Instruction::new(OpCode::Halt),
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 100_000, HashMap::new());
        let result = vm.execute().unwrap();
        assert_eq!(result.return_value, Some(42));
        assert!(!result.reverted);
    }

    #[test]
    fn test_require_fail() {
        let program = vec![
            Instruction::with_operand(OpCode::Push, 0), // false
            Instruction::new(OpCode::Require),
            Instruction::with_operand(OpCode::Push, 42),
            Instruction::new(OpCode::Halt),
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 100_000, HashMap::new());
        let result = vm.execute().unwrap();
        assert!(result.reverted);
        assert!(result.revert_reason.is_some());
    }

    #[test]
    fn test_out_of_gas() {
        let program = vec![
            Instruction::with_operand(OpCode::Push, 1),
            Instruction::with_operand(OpCode::Push, 2),
            Instruction::new(OpCode::Add),
            Instruction::new(OpCode::Halt),
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 3, HashMap::new()); // very low gas
        let result = vm.execute();
        assert!(result.is_err());
    }

    #[test]
    fn test_blockchain_context() {
        let program = vec![
            Instruction::new(OpCode::Timestamp),
            Instruction::new(OpCode::Halt),
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 100_000, HashMap::new());
        let result = vm.execute().unwrap();
        assert_eq!(result.return_value, Some(1000));
    }

    #[test]
    fn test_log_event() {
        let program = vec![
            Instruction::with_operand(OpCode::Push, 42),   // data
            Instruction::with_operand(OpCode::Push, 1001), // topic
            Instruction::with_operand(OpCode::Push, 1),    // topic_count
            Instruction::new(OpCode::Log),
            Instruction::new(OpCode::Halt),
        ];
        let mut vm = ThunderVm::new(program, default_ctx(), 100_000, HashMap::new());
        let result = vm.execute().unwrap();
        assert_eq!(result.logs.len(), 1);
        assert_eq!(result.logs[0].topics, vec![1001]);
        assert_eq!(result.logs[0].data, 42);
    }
}
