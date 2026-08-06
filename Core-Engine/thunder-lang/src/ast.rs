// ---------------------------------------------------------------------------
//  Thunder Blockchain — ThunderScript AST
// ---------------------------------------------------------------------------
//  Abstract Syntax Tree node definitions for the ThunderScript language.
// ---------------------------------------------------------------------------

/// A complete ThunderScript program (currently: one contract).
#[derive(Debug, Clone)]
pub struct Program {
    pub contract: Contract,
}

/// A contract definition.
#[derive(Debug, Clone)]
pub struct Contract {
    pub name: String,
    pub structs: Vec<StructDef>,
    pub state_vars: Vec<StateVar>,
    pub functions: Vec<Function>,
}

/// A custom struct definition.
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Parameter>, // We can reuse Parameter for name + ty
}

/// A contract state variable declaration.
#[derive(Debug, Clone)]
pub struct StateVar {
    pub name: String,
    pub ty: Type,
}

/// A function definition.
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
}

/// A function parameter.
#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

// ── Types ──────────────────────────────────────────────────────────────────

/// ThunderScript types.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    U64,
    Bool,
    Address,
    String,
    Map(Box<Type>, Box<Type>),
    Struct(String),
}

// ── Statements ─────────────────────────────────────────────────────────────

/// A statement in a function body.
#[derive(Debug, Clone)]
pub enum Statement {
    /// `let name = expr;`
    Let { name: String, value: Expression },
    /// `name = expr;` or `self.name = expr;`
    Assign {
        target: AssignTarget,
        value: Expression,
    },
    /// `if (condition) { body } else { else_body }`
    If {
        condition: Expression,
        body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    /// `while (condition) { body }`
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    /// `return expr;`
    Return { value: Option<Expression> },
    /// `require(condition, message);`
    Require {
        condition: Expression,
        message: String,
    },
    /// `emit EventName(args...);`
    Emit {
        event_name: String,
        args: Vec<Expression>,
    },
    /// An expression used as a statement (e.g., a function call).
    Expression(Expression),
}

/// The target of an assignment.
#[derive(Debug, Clone)]
pub enum AssignTarget {
    /// A local variable.
    Variable(String),
    /// `self.field`
    StateField(String),
    /// `self.field[key]`
    StateMapEntry(String, Expression),
}

// ── Expressions ────────────────────────────────────────────────────────────

/// An expression that evaluates to a value.
#[derive(Debug, Clone)]
pub enum Expression {
    /// Integer literal.
    IntLiteral(u64),
    /// Boolean literal.
    BoolLiteral(bool),
    /// String literal.
    StringLiteral(String),
    /// Variable reference.
    Variable(String),
    /// `self.field`
    StateAccess(String),
    /// `self.field[key]`
    StateMapAccess(String, Box<Expression>),
    /// Binary operation: `left op right`.
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },
    /// Unary operation: `op operand`.
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    /// Function call: `name(args...)`.
    FunctionCall { name: String, args: Vec<Expression> },
    /// Struct instantiation: `StructName { field: expr, ... }`.
    StructInit {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    /// Struct field access: `expr.field`.
    StructFieldAccess {
        target: Box<Expression>,
        field: String,
    },
}

/// Binary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
}

/// Unary operators.
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Negate,
}
