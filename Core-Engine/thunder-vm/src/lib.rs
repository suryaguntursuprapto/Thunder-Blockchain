// ---------------------------------------------------------------------------
//  Thunder Blockchain — Virtual Machine Library
// ---------------------------------------------------------------------------

pub mod gas;
pub mod opcode;
pub mod vm;

pub use vm::{ThunderVm, ExecutionContext};
pub use opcode::{Instruction, OpCode};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Compiler output: a list of instructions plus metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledContract {
    /// The contract name.
    pub name: String,
    /// The compiled bytecode (list of instructions).
    pub instructions: Vec<Instruction>,
    /// Function name → instruction index (entry point).
    pub function_table: BTreeMap<String, usize>,
    /// State variable name → storage slot index.
    pub state_slots: BTreeMap<String, u64>,
}
