// ---------------------------------------------------------------------------
//  Thunder Blockchain — ThunderScript Compiler
// ---------------------------------------------------------------------------
//  Compiles a ThunderScript AST into Thunder VM bytecode.
// ---------------------------------------------------------------------------

use std::collections::HashMap;

use thunder_vm::opcode::{Instruction, OpCode};

use crate::ast::*;

/// Compiler output: a list of instructions plus metadata.
#[derive(Debug, Clone)]
pub struct CompiledContract {
    /// The contract name.
    pub name: String,
    /// The compiled bytecode (list of instructions).
    pub instructions: Vec<Instruction>,
    /// Function name → instruction index (entry point).
    pub function_table: HashMap<String, usize>,
    /// State variable name → storage slot index.
    pub state_slots: HashMap<String, u64>,
}

/// ThunderScript → Thunder VM bytecode compiler.
pub struct Compiler {
    instructions: Vec<Instruction>,
    function_table: HashMap<String, usize>,
    state_slots: HashMap<String, u64>,
    /// Local variable name → slot index (per function).
    locals: HashMap<String, usize>,
    next_local: usize,
    next_state_slot: u64,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            function_table: HashMap::new(),
            state_slots: HashMap::new(),
            locals: HashMap::new(),
            next_local: 0,
            next_state_slot: 0,
        }
    }

    /// Compile a parsed ThunderScript program.
    pub fn compile(&mut self, program: &Program) -> Result<CompiledContract, CompileError> {
        let contract = &program.contract;

        // 1. Register state variable storage slots.
        for sv in &contract.state_vars {
            self.state_slots.insert(sv.name.clone(), self.next_state_slot);
            self.next_state_slot += 1;
        }

        // 2. Compile each function.
        for func in &contract.functions {
            self.compile_function(func)?;
        }

        // 3. If no explicit Halt, add one.
        if self.instructions.is_empty()
            || !matches!(
                self.instructions.last().map(|i| i.opcode),
                Some(OpCode::Halt) | Some(OpCode::Return)
            )
        {
            self.emit(Instruction::new(OpCode::Halt));
        }

        Ok(CompiledContract {
            name: contract.name.clone(),
            instructions: self.instructions.clone(),
            function_table: self.function_table.clone(),
            state_slots: self.state_slots.clone(),
        })
    }

    fn compile_function(&mut self, func: &Function) -> Result<(), CompileError> {
        // Reset locals for each function.
        self.locals.clear();
        self.next_local = 0;

        // Record the entry point.
        let entry = self.instructions.len();
        self.function_table.insert(func.name.clone(), entry);

        // Register parameters as local variables.
        for param in &func.params {
            let slot = self.alloc_local(&param.name);
            // Parameters are expected to be on the stack (pushed by caller).
            self.emit(Instruction::with_operand(OpCode::StoreLocal, slot as u64));
        }

        // Compile the function body.
        for stmt in &func.body {
            self.compile_statement(stmt)?;
        }

        // If the function doesn't end with a return, add one.
        if !matches!(
            self.instructions.last().map(|i| i.opcode),
            Some(OpCode::Return) | Some(OpCode::Halt) | Some(OpCode::Revert)
        ) {
            self.emit(Instruction::new(OpCode::Return));
        }

        Ok(())
    }

    // ── Statements ─────────────────────────────────────────────────────

    fn compile_statement(&mut self, stmt: &Statement) -> Result<(), CompileError> {
        match stmt {
            Statement::Let { name, value } => {
                self.compile_expression(value)?;
                let slot = self.alloc_local(name);
                self.emit(Instruction::with_operand(OpCode::StoreLocal, slot as u64));
            }
            Statement::Assign { target, value } => match target {
                AssignTarget::Variable(name) => {
                    self.compile_expression(value)?;
                    let slot = self.resolve_local(name)?;
                    self.emit(Instruction::with_operand(OpCode::StoreLocal, slot as u64));
                }
                AssignTarget::StateField(field) => {
                    let slot = self.resolve_state_slot(field)?;
                    self.emit(Instruction::with_operand(OpCode::Push, slot));
                    self.compile_expression(value)?;
                    self.emit(Instruction::new(OpCode::SStore));
                }
                AssignTarget::StateMapEntry(field, key) => {
                    // Storage key = state_slot * 1000000 + hash(key)
                    let slot = self.resolve_state_slot(field)?;
                    self.emit(Instruction::with_operand(OpCode::Push, slot));
                    self.emit(Instruction::with_operand(OpCode::Push, 1_000_000));
                    self.emit(Instruction::new(OpCode::Mul));
                    self.compile_expression(key)?;
                    self.emit(Instruction::new(OpCode::Add)); // compound key on stack
                    self.compile_expression(value)?;
                    self.emit(Instruction::new(OpCode::SStore));
                }
            },
            Statement::If {
                condition,
                body,
                else_body,
            } => {
                self.compile_expression(condition)?;

                // JumpIf to else branch (placeholder).
                let jump_else_idx = self.instructions.len();
                self.emit(Instruction::with_operand(OpCode::Push, 0)); // NOT condition
                self.emit(Instruction::new(OpCode::Not));

                let jump_to_else = self.instructions.len();
                self.emit(Instruction::with_operand(OpCode::JumpIf, 0)); // placeholder

                // Then body.
                for s in body {
                    self.compile_statement(s)?;
                }

                if !else_body.is_empty() {
                    // Jump over else body.
                    let jump_over_else = self.instructions.len();
                    self.emit(Instruction::with_operand(OpCode::Jump, 0)); // placeholder

                    // Patch jump-to-else.
                    let else_start = self.instructions.len();
                    self.instructions[jump_to_else].operand = else_start as u64;

                    for s in else_body {
                        self.compile_statement(s)?;
                    }

                    // Patch jump-over-else.
                    let after_else = self.instructions.len();
                    self.instructions[jump_over_else].operand = after_else as u64;
                } else {
                    // Patch jump-to-else to skip the body.
                    let after_if = self.instructions.len();
                    self.instructions[jump_to_else].operand = after_if as u64;
                }
            }
            Statement::While { condition, body } => {
                let loop_start = self.instructions.len();
                self.compile_expression(condition)?;

                // NOT condition → JumpIf to after loop.
                self.emit(Instruction::new(OpCode::Not));
                let jump_exit = self.instructions.len();
                self.emit(Instruction::with_operand(OpCode::JumpIf, 0)); // placeholder

                for s in body {
                    self.compile_statement(s)?;
                }

                // Jump back to loop start.
                self.emit(Instruction::with_operand(OpCode::Jump, loop_start as u64));

                // Patch exit jump.
                let after_loop = self.instructions.len();
                self.instructions[jump_exit].operand = after_loop as u64;
            }
            Statement::Return { value } => {
                if let Some(expr) = value {
                    self.compile_expression(expr)?;
                }
                self.emit(Instruction::new(OpCode::Return));
            }
            Statement::Require { condition, message } => {
                self.compile_expression(condition)?;
                self.emit(Instruction::new(OpCode::Require));
            }
            Statement::Emit { event_name, args } => {
                // Push data (last arg or 0).
                if let Some(last) = args.last() {
                    self.compile_expression(last)?;
                } else {
                    self.emit(Instruction::with_operand(OpCode::Push, 0));
                }

                // Push topics (all args except the last are topics).
                let topic_args = if args.len() > 1 {
                    &args[..args.len() - 1]
                } else {
                    &[]
                };
                for arg in topic_args {
                    self.compile_expression(arg)?;
                }

                // Push topic count.
                self.emit(Instruction::with_operand(
                    OpCode::Push,
                    topic_args.len() as u64,
                ));
                self.emit(Instruction::new(OpCode::Log));
            }
            Statement::Expression(expr) => {
                self.compile_expression(expr)?;
                // Discard the result.
                self.emit(Instruction::new(OpCode::Pop));
            }
        }
        Ok(())
    }

    // ── Expressions ────────────────────────────────────────────────────

    fn compile_expression(&mut self, expr: &Expression) -> Result<(), CompileError> {
        match expr {
            Expression::IntLiteral(val) => {
                self.emit(Instruction::with_operand(OpCode::Push, *val));
            }
            Expression::BoolLiteral(val) => {
                self.emit(Instruction::with_operand(
                    OpCode::Push,
                    if *val { 1 } else { 0 },
                ));
            }
            Expression::StringLiteral(val) => {
                // Strings are stored as their hash for simplicity.
                let hash_val = {
                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    hasher.update(val.as_bytes());
                    let result = hasher.finalize();
                    u64::from_le_bytes(result[..8].try_into().unwrap())
                };
                self.emit(Instruction::with_operand(OpCode::Push, hash_val));
            }
            Expression::Variable(name) => {
                let slot = self.resolve_local(name)?;
                self.emit(Instruction::with_operand(OpCode::LoadLocal, slot as u64));
            }
            Expression::StateAccess(field) => {
                let slot = self.resolve_state_slot(field)?;
                self.emit(Instruction::with_operand(OpCode::Push, slot));
                self.emit(Instruction::new(OpCode::SLoad));
            }
            Expression::StateMapAccess(field, key) => {
                let slot = self.resolve_state_slot(field)?;
                self.emit(Instruction::with_operand(OpCode::Push, slot));
                self.emit(Instruction::with_operand(OpCode::Push, 1_000_000));
                self.emit(Instruction::new(OpCode::Mul));
                self.compile_expression(key)?;
                self.emit(Instruction::new(OpCode::Add));
                self.emit(Instruction::new(OpCode::SLoad));
            }
            Expression::BinaryOp { left, op, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                let opcode = match op {
                    BinaryOperator::Add => OpCode::Add,
                    BinaryOperator::Sub => OpCode::Sub,
                    BinaryOperator::Mul => OpCode::Mul,
                    BinaryOperator::Div => OpCode::Div,
                    BinaryOperator::Mod => OpCode::Mod,
                    BinaryOperator::Eq => OpCode::Eq,
                    BinaryOperator::Neq => OpCode::Neq,
                    BinaryOperator::Lt => OpCode::Lt,
                    BinaryOperator::Gt => OpCode::Gt,
                    BinaryOperator::Lte => OpCode::Lte,
                    BinaryOperator::Gte => OpCode::Gte,
                    BinaryOperator::And => OpCode::And,
                    BinaryOperator::Or => OpCode::Or,
                };
                self.emit(Instruction::new(opcode));
            }
            Expression::UnaryOp { op, operand } => {
                self.compile_expression(operand)?;
                match op {
                    UnaryOperator::Not => {
                        self.emit(Instruction::new(OpCode::Not));
                    }
                    UnaryOperator::Negate => {
                        // 0 - operand
                        self.emit(Instruction::with_operand(OpCode::Push, 0));
                        self.emit(Instruction::new(OpCode::Swap));
                        self.emit(Instruction::new(OpCode::Sub));
                    }
                }
            }
            Expression::FunctionCall { name, args } => {
                // Built-in functions.
                match name.as_str() {
                    "caller" => {
                        self.emit(Instruction::new(OpCode::Caller));
                    }
                    "balance" => {
                        if let Some(arg) = args.first() {
                            self.compile_expression(arg)?;
                        }
                        self.emit(Instruction::new(OpCode::Balance));
                    }
                    "timestamp" => {
                        self.emit(Instruction::new(OpCode::Timestamp));
                    }
                    "block_height" => {
                        self.emit(Instruction::new(OpCode::BlockHeight));
                    }
                    "self_address" => {
                        self.emit(Instruction::new(OpCode::SelfAddress));
                    }
                    "hash" => {
                        if let Some(arg) = args.first() {
                            self.compile_expression(arg)?;
                        }
                        self.emit(Instruction::new(OpCode::Hash));
                    }
                    _ => {
                        // User-defined function call.
                        // Push arguments in reverse order.
                        for arg in args.iter().rev() {
                            self.compile_expression(arg)?;
                        }
                        // Call will be resolved at link time; for now use a
                        // placeholder that the entry table can resolve.
                        self.emit(Instruction::with_operand(OpCode::Call, 0));
                        // Mark this instruction for later resolution.
                    }
                }
            }
        }
        Ok(())
    }

    // ── Helpers ────────────────────────────────────────────────────────

    fn emit(&mut self, instr: Instruction) {
        self.instructions.push(instr);
    }

    fn alloc_local(&mut self, name: &str) -> usize {
        let slot = self.next_local;
        self.locals.insert(name.to_string(), slot);
        self.next_local += 1;
        slot
    }

    fn resolve_local(&self, name: &str) -> Result<usize, CompileError> {
        self.locals
            .get(name)
            .copied()
            .ok_or_else(|| CompileError::UndefinedVariable(name.to_string()))
    }

    fn resolve_state_slot(&self, name: &str) -> Result<u64, CompileError> {
        self.state_slots
            .get(name)
            .copied()
            .ok_or_else(|| CompileError::UndefinedStateVar(name.to_string()))
    }
}

impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

// ── Errors ─────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum CompileError {
    #[error("undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("undefined state variable: {0}")]
    UndefinedStateVar(String),

    #[error("compilation error: {0}")]
    General(String),
}

// ── Public compile function ────────────────────────────────────────────────

/// Convenience function: compile ThunderScript source → bytecode.
pub fn compile_source(source: &str) -> Result<CompiledContract, String> {
    let mut lexer = crate::lexer::Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = crate::parser::Parser::new(tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut compiler = Compiler::new();
    compiler.compile(&program).map_err(|e| e.to_string())
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_empty_contract() {
        let src = "contract Empty { }";
        let compiled = compile_source(src).unwrap();
        assert_eq!(compiled.name, "Empty");
        // Should have at least a Halt instruction.
        assert!(!compiled.instructions.is_empty());
    }

    #[test]
    fn test_compile_simple_function() {
        let src = r#"
contract Math {
    fn add(a: u64, b: u64) -> u64 {
        return a + b;
    }
}
        "#;
        let compiled = compile_source(src).unwrap();
        assert!(compiled.function_table.contains_key("add"));
        // The instructions should contain Push, LoadLocal, Add, Return, etc.
        assert!(compiled.instructions.len() > 3);
    }

    #[test]
    fn test_compile_state_access() {
        let src = r#"
contract Counter {
    state count: u64;

    fn increment() {
        self.count = self.count + 1;
    }
}
        "#;
        let compiled = compile_source(src).unwrap();
        assert!(compiled.state_slots.contains_key("count"));
        let has_sstore = compiled
            .instructions
            .iter()
            .any(|i| i.opcode == OpCode::SStore);
        assert!(has_sstore, "should contain SStore for self.count = ...");
    }

    #[test]
    fn test_compile_if_else() {
        let src = r#"
contract C {
    fn check(x: u64) -> u64 {
        if (x > 10) {
            return 1;
        } else {
            return 0;
        }
    }
}
        "#;
        let compiled = compile_source(src).unwrap();
        let has_jump = compiled
            .instructions
            .iter()
            .any(|i| matches!(i.opcode, OpCode::Jump | OpCode::JumpIf));
        assert!(has_jump, "should contain jump instructions for if/else");
    }

    #[test]
    fn test_compile_while_loop() {
        let src = r#"
contract C {
    fn count_to(n: u64) -> u64 {
        let i = 0;
        while (i < n) {
            i = i + 1;
        }
        return i;
    }
}
        "#;
        let compiled = compile_source(src).unwrap();
        let jumps: Vec<_> = compiled
            .instructions
            .iter()
            .filter(|i| matches!(i.opcode, OpCode::Jump | OpCode::JumpIf))
            .collect();
        assert!(jumps.len() >= 2, "while loop needs at least 2 jumps");
    }

    #[test]
    fn test_compile_require() {
        let src = r#"
contract C {
    fn safe(x: u64) {
        require(x > 0, "must be positive");
    }
}
        "#;
        let compiled = compile_source(src).unwrap();
        let has_require = compiled
            .instructions
            .iter()
            .any(|i| i.opcode == OpCode::Require);
        assert!(has_require);
    }

    #[test]
    fn test_compile_emit() {
        let src = r#"
contract C {
    fn fire() {
        emit Transfer(1, 2, 100);
    }
}
        "#;
        let compiled = compile_source(src).unwrap();
        let has_log = compiled
            .instructions
            .iter()
            .any(|i| i.opcode == OpCode::Log);
        assert!(has_log);
    }

    #[test]
    fn test_compile_builtin_caller() {
        let src = r#"
contract C {
    fn who() -> u64 {
        return caller();
    }
}
        "#;
        let compiled = compile_source(src).unwrap();
        let has_caller = compiled
            .instructions
            .iter()
            .any(|i| i.opcode == OpCode::Caller);
        assert!(has_caller);
    }
}
