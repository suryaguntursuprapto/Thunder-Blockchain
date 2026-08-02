// ---------------------------------------------------------------------------
//  Thunder Blockchain — ThunderScript Parser
// ---------------------------------------------------------------------------
//  Recursive-descent parser: transforms a token stream into an AST.
// ---------------------------------------------------------------------------

use crate::ast::*;
use crate::lexer::{Token, TokenKind};

/// ThunderScript parser.
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Create a new parser from a token stream.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parse a full ThunderScript program.
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let contract = self.parse_contract()?;
        Ok(Program { contract })
    }

    // ── Contract ───────────────────────────────────────────────────────

    fn parse_contract(&mut self) -> Result<Contract, ParseError> {
        self.expect(TokenKind::Contract)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LeftBrace)?;

        let mut state_vars = Vec::new();
        let mut functions = Vec::new();

        while !self.check(TokenKind::RightBrace) && !self.check(TokenKind::Eof) {
            if self.check(TokenKind::State) {
                state_vars.push(self.parse_state_var()?);
            } else if self.check(TokenKind::Fn) {
                functions.push(self.parse_function()?);
            } else {
                return Err(self.error("expected 'state' or 'fn'"));
            }
        }

        self.expect(TokenKind::RightBrace)?;

        Ok(Contract {
            name,
            state_vars,
            functions,
        })
    }

    // ── State Variables ────────────────────────────────────────────────

    fn parse_state_var(&mut self) -> Result<StateVar, ParseError> {
        self.expect(TokenKind::State)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(StateVar { name, ty })
    }

    // ── Functions ──────────────────────────────────────────────────────

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        self.expect(TokenKind::Fn)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::LeftParen)?;

        let mut params = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                let param_name = self.expect_identifier()?;
                self.expect(TokenKind::Colon)?;
                let ty = self.parse_type()?;
                params.push(Parameter {
                    name: param_name,
                    ty,
                });
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RightParen)?;

        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };

        self.expect(TokenKind::LeftBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RightBrace)?;

        Ok(Function {
            name,
            params,
            return_type,
            body,
        })
    }

    // ── Types ──────────────────────────────────────────────────────────

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        let token = self.advance();
        match &token.kind {
            TokenKind::U64 => Ok(Type::U64),
            TokenKind::Bool => Ok(Type::Bool),
            TokenKind::Address => Ok(Type::Address),
            TokenKind::StringType => Ok(Type::String),
            TokenKind::Map => {
                self.expect(TokenKind::Lt)?;
                let key_type = self.parse_type()?;
                self.expect(TokenKind::Comma)?;
                let value_type = self.parse_type()?;
                self.expect(TokenKind::Gt)?;
                Ok(Type::Map(Box::new(key_type), Box::new(value_type)))
            }
            _ => Err(self.error_at(&token, "expected type")),
        }
    }

    // ── Statements ─────────────────────────────────────────────────────

    fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut stmts = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.check(TokenKind::Eof) {
            stmts.push(self.parse_statement()?);
        }
        Ok(stmts)
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        if self.check(TokenKind::Let) {
            self.parse_let()
        } else if self.check(TokenKind::If) {
            self.parse_if()
        } else if self.check(TokenKind::While) {
            self.parse_while()
        } else if self.check(TokenKind::Return) {
            self.parse_return()
        } else if self.check(TokenKind::Require) {
            self.parse_require()
        } else if self.check(TokenKind::Emit) {
            self.parse_emit()
        } else if self.check(TokenKind::Self_) {
            self.parse_self_statement()
        } else if self.check_identifier() {
            self.parse_assign_or_expression()
        } else {
            let expr = self.parse_expression()?;
            self.expect(TokenKind::Semicolon)?;
            Ok(Statement::Expression(expr))
        }
    }

    fn parse_let(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Let)?;
        let name = self.expect_identifier()?;
        self.expect(TokenKind::Assign)?;
        let value = self.parse_expression()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Statement::Let { name, value })
    }

    fn parse_if(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::If)?;
        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;
        self.expect(TokenKind::LeftBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RightBrace)?;

        let else_body = if self.match_token(TokenKind::Else) {
            self.expect(TokenKind::LeftBrace)?;
            let stmts = self.parse_block()?;
            self.expect(TokenKind::RightBrace)?;
            stmts
        } else {
            Vec::new()
        };

        Ok(Statement::If {
            condition,
            body,
            else_body,
        })
    }

    fn parse_while(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::While)?;
        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::RightParen)?;
        self.expect(TokenKind::LeftBrace)?;
        let body = self.parse_block()?;
        self.expect(TokenKind::RightBrace)?;
        Ok(Statement::While { condition, body })
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Return)?;
        let value = if self.check(TokenKind::Semicolon) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        self.expect(TokenKind::Semicolon)?;
        Ok(Statement::Return { value })
    }

    fn parse_require(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Require)?;
        self.expect(TokenKind::LeftParen)?;
        let condition = self.parse_expression()?;
        self.expect(TokenKind::Comma)?;
        let message = self.expect_string()?;
        self.expect(TokenKind::RightParen)?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Statement::Require { condition, message })
    }

    fn parse_emit(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Emit)?;
        let event_name = self.expect_identifier()?;
        self.expect(TokenKind::LeftParen)?;
        let mut args = Vec::new();
        if !self.check(TokenKind::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenKind::RightParen)?;
        self.expect(TokenKind::Semicolon)?;
        Ok(Statement::Emit { event_name, args })
    }

    fn parse_self_statement(&mut self) -> Result<Statement, ParseError> {
        self.expect(TokenKind::Self_)?;
        self.expect(TokenKind::Dot)?;
        let field = self.expect_identifier()?;

        if self.match_token(TokenKind::LeftBracket) {
            // self.field[key] = value;
            let key = self.parse_expression()?;
            self.expect(TokenKind::RightBracket)?;
            self.expect(TokenKind::Assign)?;
            let value = self.parse_expression()?;
            self.expect(TokenKind::Semicolon)?;
            Ok(Statement::Assign {
                target: AssignTarget::StateMapEntry(field, key),
                value,
            })
        } else {
            // self.field = value;
            self.expect(TokenKind::Assign)?;
            let value = self.parse_expression()?;
            self.expect(TokenKind::Semicolon)?;
            Ok(Statement::Assign {
                target: AssignTarget::StateField(field),
                value,
            })
        }
    }

    fn parse_assign_or_expression(&mut self) -> Result<Statement, ParseError> {
        let name = self.expect_identifier()?;

        if self.match_token(TokenKind::Assign) {
            let value = self.parse_expression()?;
            self.expect(TokenKind::Semicolon)?;
            Ok(Statement::Assign {
                target: AssignTarget::Variable(name),
                value,
            })
        } else if self.check(TokenKind::LeftParen) {
            // Function call as statement.
            self.expect(TokenKind::LeftParen)?;
            let mut args = Vec::new();
            if !self.check(TokenKind::RightParen) {
                loop {
                    args.push(self.parse_expression()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.expect(TokenKind::RightParen)?;
            self.expect(TokenKind::Semicolon)?;
            Ok(Statement::Expression(Expression::FunctionCall {
                name,
                args,
            }))
        } else {
            Err(self.error("expected '=' or '(' after identifier"))
        }
    }

    // ── Expressions (precedence climbing) ──────────────────────────────

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_and()?;
        while self.match_token(TokenKind::Or) {
            let right = self.parse_and()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_equality()?;
        while self.match_token(TokenKind::And) {
            let right = self.parse_equality()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;
        loop {
            if self.match_token(TokenKind::Eq) {
                let right = self.parse_comparison()?;
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::Eq,
                    right: Box::new(right),
                };
            } else if self.match_token(TokenKind::Neq) {
                let right = self.parse_comparison()?;
                left = Expression::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::Neq,
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_additive()?;
        loop {
            let op = if self.match_token(TokenKind::Lt) {
                BinaryOperator::Lt
            } else if self.match_token(TokenKind::Gt) {
                BinaryOperator::Gt
            } else if self.match_token(TokenKind::Lte) {
                BinaryOperator::Lte
            } else if self.match_token(TokenKind::Gte) {
                BinaryOperator::Gte
            } else {
                break;
            };
            let right = self.parse_additive()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = if self.match_token(TokenKind::Plus) {
                BinaryOperator::Add
            } else if self.match_token(TokenKind::Minus) {
                BinaryOperator::Sub
            } else {
                break;
            };
            let right = self.parse_multiplicative()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = if self.match_token(TokenKind::Star) {
                BinaryOperator::Mul
            } else if self.match_token(TokenKind::Slash) {
                BinaryOperator::Div
            } else if self.match_token(TokenKind::Percent) {
                BinaryOperator::Mod
            } else {
                break;
            };
            let right = self.parse_unary()?;
            left = Expression::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if self.match_token(TokenKind::Not) {
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOp {
                op: UnaryOperator::Not,
                operand: Box::new(operand),
            });
        }
        if self.match_token(TokenKind::Minus) {
            let operand = self.parse_unary()?;
            return Ok(Expression::UnaryOp {
                op: UnaryOperator::Negate,
                operand: Box::new(operand),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let token = self.peek().clone();

        match &token.kind {
            TokenKind::IntLiteral(val) => {
                let val = *val;
                self.advance();
                Ok(Expression::IntLiteral(val))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::BoolLiteral(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::BoolLiteral(false))
            }
            TokenKind::StringLiteral(val) => {
                let val = val.clone();
                self.advance();
                Ok(Expression::StringLiteral(val))
            }
            TokenKind::Self_ => {
                self.advance();
                self.expect(TokenKind::Dot)?;
                let field = self.expect_identifier()?;
                if self.match_token(TokenKind::LeftBracket) {
                    let key = self.parse_expression()?;
                    self.expect(TokenKind::RightBracket)?;
                    Ok(Expression::StateMapAccess(field, Box::new(key)))
                } else {
                    Ok(Expression::StateAccess(field))
                }
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if self.check(TokenKind::LeftParen) {
                    // Function call.
                    self.expect(TokenKind::LeftParen)?;
                    let mut args = Vec::new();
                    if !self.check(TokenKind::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token(TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.expect(TokenKind::RightParen)?;
                    Ok(Expression::FunctionCall { name, args })
                } else {
                    Ok(Expression::Variable(name))
                }
            }
            TokenKind::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RightParen)?;
                Ok(expr)
            }
            _ => Err(self.error_at(&token, "expected expression")),
        }
    }

    // ── Token helpers ──────────────────────────────────────────────────

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.pos)
            .unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn check(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(&kind)
    }

    fn check_identifier(&self) -> bool {
        matches!(self.peek().kind, TokenKind::Identifier(_))
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: TokenKind) -> Result<Token, ParseError> {
        if self.check(expected.clone()) {
            Ok(self.advance())
        } else {
            Err(self.error(&format!(
                "expected {:?}, got {:?}",
                expected,
                self.peek().kind
            )))
        }
    }

    fn expect_identifier(&mut self) -> Result<String, ParseError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::Identifier(name) => Ok(name),
            _ => Err(self.error_at(&tok, "expected identifier")),
        }
    }

    fn expect_string(&mut self) -> Result<String, ParseError> {
        let tok = self.advance();
        match tok.kind {
            TokenKind::StringLiteral(val) => Ok(val),
            _ => Err(self.error_at(&tok, "expected string literal")),
        }
    }

    fn error(&self, msg: &str) -> ParseError {
        let tok = self.peek();
        ParseError {
            message: msg.to_string(),
            line: tok.line,
            column: tok.column,
        }
    }

    fn error_at(&self, token: &Token, msg: &str) -> ParseError {
        ParseError {
            message: format!("{} (got '{}')", msg, token.lexeme),
            line: token.line,
            column: token.column,
        }
    }
}

// ── Error ──────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
#[error("Parse error at {line}:{column}: {message}")]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn parse_source(src: &str) -> Result<Program, ParseError> {
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().expect("lexer failed");
        let mut parser = Parser::new(tokens);
        parser.parse()
    }

    #[test]
    fn test_empty_contract() {
        let program = parse_source("contract Empty { }").unwrap();
        assert_eq!(program.contract.name, "Empty");
        assert!(program.contract.state_vars.is_empty());
        assert!(program.contract.functions.is_empty());
    }

    #[test]
    fn test_state_vars() {
        let program = parse_source(
            "contract Token {
                state owner: address;
                state total: u64;
            }",
        )
        .unwrap();
        assert_eq!(program.contract.state_vars.len(), 2);
        assert_eq!(program.contract.state_vars[0].name, "owner");
        assert_eq!(program.contract.state_vars[0].ty, Type::Address);
        assert_eq!(program.contract.state_vars[1].name, "total");
        assert_eq!(program.contract.state_vars[1].ty, Type::U64);
    }

    #[test]
    fn test_simple_function() {
        let program = parse_source(
            "contract C {
                fn add(a: u64, b: u64) -> u64 {
                    return a + b;
                }
            }",
        )
        .unwrap();
        assert_eq!(program.contract.functions.len(), 1);
        let f = &program.contract.functions[0];
        assert_eq!(f.name, "add");
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.return_type, Some(Type::U64));
    }

    #[test]
    fn test_if_else() {
        let program = parse_source(
            "contract C {
                fn check(x: u64) {
                    if (x > 10) {
                        return;
                    } else {
                        return;
                    }
                }
            }",
        )
        .unwrap();
        let body = &program.contract.functions[0].body;
        assert!(matches!(body[0], Statement::If { .. }));
    }

    #[test]
    fn test_while_loop() {
        let program = parse_source(
            "contract C {
                fn loop_fn() {
                    let i = 0;
                    while (i < 10) {
                        i = i + 1;
                    }
                }
            }",
        )
        .unwrap();
        let body = &program.contract.functions[0].body;
        assert!(matches!(body[0], Statement::Let { .. }));
        assert!(matches!(body[1], Statement::While { .. }));
    }

    #[test]
    fn test_require_statement() {
        let program = parse_source(
            r#"contract C {
                fn safe(x: u64) {
                    require(x > 0, "must be positive");
                }
            }"#,
        )
        .unwrap();
        let body = &program.contract.functions[0].body;
        assert!(matches!(body[0], Statement::Require { .. }));
    }

    #[test]
    fn test_self_access() {
        let program = parse_source(
            "contract C {
                state val: u64;
                fn set() {
                    self.val = 42;
                }
            }",
        )
        .unwrap();
        let body = &program.contract.functions[0].body;
        assert!(matches!(
            body[0],
            Statement::Assign {
                target: AssignTarget::StateField(_),
                ..
            }
        ));
    }

    #[test]
    fn test_map_type_and_access() {
        let program = parse_source(
            "contract C {
                state balances: map<address, u64>;
                fn get(addr: address) -> u64 {
                    return self.balances[addr];
                }
            }",
        )
        .unwrap();
        let sv = &program.contract.state_vars[0];
        assert!(matches!(sv.ty, Type::Map(_, _)));
    }

    #[test]
    fn test_emit_statement() {
        let program = parse_source(
            "contract C {
                fn fire() {
                    emit Transfer(1, 2, 100);
                }
            }",
        )
        .unwrap();
        let body = &program.contract.functions[0].body;
        assert!(matches!(body[0], Statement::Emit { .. }));
    }

    #[test]
    fn test_function_call_expression() {
        let program = parse_source(
            "contract C {
                fn test() {
                    let x = caller();
                }
            }",
        )
        .unwrap();
        let body = &program.contract.functions[0].body;
        assert!(matches!(body[0], Statement::Let { .. }));
    }

    #[test]
    fn test_full_token_contract() {
        let src = r#"
contract Token {
    state owner: address;
    state balances: map<address, u64>;

    fn init() {
        self.owner = caller();
        self.balances[caller()] = 1000000;
    }

    fn transfer(to: address, amount: u64) {
        let sender_bal = self.balances[caller()];
        require(sender_bal >= amount, "Insufficient balance");
        self.balances[caller()] = sender_bal - amount;
        self.balances[to] = self.balances[to] + amount;
        emit Transfer(caller(), to, amount);
    }

    fn balance_of(addr: address) -> u64 {
        return self.balances[addr];
    }
}
        "#;
        let program = parse_source(src).unwrap();
        assert_eq!(program.contract.name, "Token");
        assert_eq!(program.contract.state_vars.len(), 2);
        assert_eq!(program.contract.functions.len(), 3);
    }
}
