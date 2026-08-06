// ---------------------------------------------------------------------------
//  Thunder Blockchain — Gas Metering
// ---------------------------------------------------------------------------
//  Defines the cost of each opcode and tracks gas consumption during
//  smart contract execution.
// ---------------------------------------------------------------------------

use crate::opcode::OpCode;

/// Gas cost table — returns the gas cost for a given opcode.
pub fn gas_cost(opcode: OpCode) -> u64 {
    match opcode {
        // Stack operations — very cheap.
        OpCode::Push | OpCode::Pop | OpCode::Dup | OpCode::Swap => 2,

        // Arithmetic — cheap.
        OpCode::Add | OpCode::Sub | OpCode::Mul | OpCode::Mod => 3,
        OpCode::Div => 5, // Division is slightly more expensive.

        // Comparison — cheap.
        OpCode::Eq | OpCode::Neq | OpCode::Lt | OpCode::Gt | OpCode::Lte | OpCode::Gte => 3,

        // Logic — cheap.
        OpCode::And | OpCode::Or | OpCode::Not => 3,

        // Control flow.
        OpCode::Jump | OpCode::JumpIf => 8,
        OpCode::Call => 40,
        OpCode::Return => 5,
        OpCode::Halt => 0,
        OpCode::Revert => 0,

        // Memory (local variables & linear).
        OpCode::LoadLocal | OpCode::StoreLocal | OpCode::MLoad | OpCode::MStore => 3,

        // Persistent storage — expensive (disk I/O).
        OpCode::SLoad => 200,
        OpCode::SStore => 5_000,

        // Blockchain context — moderate.
        OpCode::Caller | OpCode::Timestamp | OpCode::BlockHeight | OpCode::SelfAddress => 2,
        OpCode::Balance => 100,
        OpCode::Transfer => 9_000,

        // Data & Cryptography.
        OpCode::PushBytes => 3,
        OpCode::Hash | OpCode::Keccak256 => 30,
        OpCode::VerifySig => 3_000,

        // Events.
        OpCode::Log => 375,

        // Assertions.
        OpCode::Require => 5,
    }
}

/// Tracks gas consumption during VM execution.
#[derive(Debug, Clone)]
pub struct GasMeter {
    /// Total gas budget for this execution.
    pub gas_limit: u64,
    /// Gas consumed so far.
    pub gas_used: u64,
    /// Current base fee per unit of gas (EIP-1559).
    pub base_fee: u64,
}

impl GasMeter {
    /// Create a new gas meter with the given limit and base fee.
    pub fn new(gas_limit: u64, base_fee: u64) -> Self {
        Self {
            gas_limit,
            gas_used: 0,
            base_fee,
        }
    }

    /// Consume `amount` gas.  Returns `Err` if the limit is exceeded.
    pub fn consume(&mut self, amount: u64) -> Result<(), GasError> {
        self.gas_used = self.gas_used.saturating_add(amount);
        if self.gas_used > self.gas_limit {
            Err(GasError::OutOfGas {
                limit: self.gas_limit,
                used: self.gas_used,
            })
        } else {
            Ok(())
        }
    }

    /// Consume gas for a specific opcode.
    pub fn consume_opcode(&mut self, opcode: OpCode) -> Result<(), GasError> {
        self.consume(gas_cost(opcode))
    }

    /// Remaining gas units.
    pub fn remaining(&self) -> u64 {
        self.gas_limit.saturating_sub(self.gas_used)
    }

    /// Gas units used so far.
    pub fn used(&self) -> u64 {
        self.gas_used
    }

    /// Total fee (in minimal network token units) for the consumed gas.
    pub fn fee(&self) -> u64 {
        self.gas_used.saturating_mul(self.base_fee)
    }
}

/// Dynamic Gas Market (EIP-1559 Style)
#[derive(Debug, Clone)]
pub struct GasMarket {
    pub current_base_fee: u64,
    pub target_gas_per_block: u64,
    pub max_gas_per_block: u64,
    pub base_fee_max_change_denominator: u64,
}

impl GasMarket {
    pub fn new(initial_base_fee: u64) -> Self {
        Self {
            current_base_fee: initial_base_fee,
            target_gas_per_block: 15_000_000,
            max_gas_per_block: 30_000_000,
            base_fee_max_change_denominator: 8, // Max 12.5% change per block
        }
    }

    /// Calculate the next block's base fee based on the gas used in the current block.
    pub fn next_base_fee(&self, block_gas_used: u64) -> u64 {
        if block_gas_used == self.target_gas_per_block {
            return self.current_base_fee;
        }

        if block_gas_used > self.target_gas_per_block {
            let gas_used_delta = block_gas_used - self.target_gas_per_block;
            let fee_delta = std::cmp::max(
                (self.current_base_fee * gas_used_delta) / self.target_gas_per_block / self.base_fee_max_change_denominator,
                1,
            );
            self.current_base_fee + fee_delta
        } else {
            let gas_used_delta = self.target_gas_per_block - block_gas_used;
            let fee_delta = (self.current_base_fee * gas_used_delta) / self.target_gas_per_block / self.base_fee_max_change_denominator;
            self.current_base_fee.saturating_sub(fee_delta)
        }
    }
}

/// Out-of-gas error.
#[derive(Debug, thiserror::Error)]
pub enum GasError {
    #[error("out of gas: limit {limit}, used {used}")]
    OutOfGas { limit: u64, used: u64 },
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_meter_basic() {
        let mut meter = GasMeter::new(100, 10);
        assert!(meter.consume(50).is_ok());
        assert_eq!(meter.remaining(), 50);
        assert_eq!(meter.used(), 50);
        assert_eq!(meter.fee(), 500);
    }

    #[test]
    fn test_gas_meter_exact_limit() {
        let mut meter = GasMeter::new(100, 10);
        assert!(meter.consume(100).is_ok());
        assert_eq!(meter.remaining(), 0);
    }

    #[test]
    fn test_gas_meter_out_of_gas() {
        let mut meter = GasMeter::new(100, 10);
        assert!(meter.consume(101).is_err());
    }

    #[test]
    fn test_opcode_gas_cost() {
        assert!(gas_cost(OpCode::SStore) > gas_cost(OpCode::Add));
        assert!(gas_cost(OpCode::Transfer) > gas_cost(OpCode::Caller));
    }

    #[test]
    fn test_consume_opcode() {
        let mut meter = GasMeter::new(10, 10);
        assert!(meter.consume_opcode(OpCode::Push).is_ok()); // 2
        assert!(meter.consume_opcode(OpCode::Push).is_ok()); // 4
        assert!(meter.consume_opcode(OpCode::Add).is_ok()); // 7
        assert_eq!(meter.used(), 7);
    }

    #[test]
    fn test_gas_market() {
        let market = GasMarket::new(100);
        // If gas used == target, base fee is unchanged.
        assert_eq!(market.next_base_fee(15_000_000), 100);
        // If gas used > target, base fee increases.
        assert!(market.next_base_fee(20_000_000) > 100);
        // If gas used < target, base fee decreases.
        assert!(market.next_base_fee(10_000_000) < 100);
    }
}
