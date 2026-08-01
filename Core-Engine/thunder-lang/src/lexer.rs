// ---------------------------------------------------------------------------
//  Thunder Blockchain — ThunderScript Lexer
// ---------------------------------------------------------------------------
//  Tokenises ThunderScript source code into a stream of tokens.
// ---------------------------------------------------------------------------

use std::fmt;

// ── Token Types ────────────────────────────────────────────────────────────

/// A token produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

/// All possible token types in ThunderScript.
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ── Keywords ───────────────────────────────────────────────────────
    Contract,
    Fn,
    Let,
    If,
    Else,
    While,
    Return,
    State,
    Emit,
    Require,
    Self_,       // `self`
    True,
    False,

    // ── Types ──────────────────────────────────────────────────────────
    U64,
    Bool,
    Address,
    StringType,
    Map,

    // ── Literals ───────────────────────────────────────────────────────
    IntLiteral(u64),
    StringLiteral(String),

    // ── Identifiers ────────────────────────────────────────────────────
    Identifier(String),

    // ── Operators ──────────────────────────────────────────────────────
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /
    Percent,     // %
    Eq,          // ==
    Neq,         // !=
    Lt,          // <
    Gt,          // >
    Lte,         // <=
    Gte,         // >=
    And,         // &&
    Or,          // ||
    Not,         // !
    Assign,      // =

    // ── Delimiters ─────────────────────────────────────────────────────
    LeftParen,   // (
    RightParen,  // )
    LeftBrace,   // {
    RightBrace,  // }
    LeftBracket, // [
    RightBracket,// ]
    Comma,       // ,
    Semicolon,   // ;
    Colon,       // :
    Dot,         // .
    Arrow,       // ->

    // ── Special ────────────────────────────────────────────────────────
    Eof,
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Contract => write!(f, "contract"),
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Identifier(name) => write!(f, "{}", name),
            TokenKind::IntLiteral(val) => write!(f, "{}", val),
            TokenKind::StringLiteral(val) => write!(f, "\"{}\"", val),
            TokenKind::Eof => write!(f, "EOF"),
            other => write!(f, "{:?}", other),
        }
    }
}

// ── Lexer ──────────────────────────────────────────────────────────────────

/// ThunderScript lexer.
pub struct Lexer {
    source: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Create a new lexer for the given source string.
    pub fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenise the entire source into a vector of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();
            self.skip_comments();
            self.skip_whitespace();

            if self.is_at_end() {
                tokens.push(self.make_token(TokenKind::Eof, ""));
                break;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        Ok(tokens)
    }

    /// Read the next token.
    fn next_token(&mut self) -> Result<Token, LexerError> {
        let ch = self.peek();

        // Numbers
        if ch.is_ascii_digit() {
            return self.read_number();
        }

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return Ok(self.read_identifier());
        }

        // String literals
        if ch == '"' {
            return self.read_string();
        }

        // Operators and delimiters
        match ch {
            '+' => Ok(self.single_char_token(TokenKind::Plus)),
            '*' => Ok(self.single_char_token(TokenKind::Star)),
            '%' => Ok(self.single_char_token(TokenKind::Percent)),
            '(' => Ok(self.single_char_token(TokenKind::LeftParen)),
            ')' => Ok(self.single_char_token(TokenKind::RightParen)),
            '{' => Ok(self.single_char_token(TokenKind::LeftBrace)),
            '}' => Ok(self.single_char_token(TokenKind::RightBrace)),
            '[' => Ok(self.single_char_token(TokenKind::LeftBracket)),
            ']' => Ok(self.single_char_token(TokenKind::RightBracket)),
            ',' => Ok(self.single_char_token(TokenKind::Comma)),
            ';' => Ok(self.single_char_token(TokenKind::Semicolon)),
            ':' => Ok(self.single_char_token(TokenKind::Colon)),
            '.' => Ok(self.single_char_token(TokenKind::Dot)),

            '-' => {
                if self.peek_next() == '>' {
                    let tok = self.make_token(TokenKind::Arrow, "->");
                    self.advance();
                    self.advance();
                    Ok(tok)
                } else {
                    Ok(self.single_char_token(TokenKind::Minus))
                }
            }

            '=' => {
                if self.peek_next() == '=' {
                    let tok = self.make_token(TokenKind::Eq, "==");
                    self.advance();
                    self.advance();
                    Ok(tok)
                } else {
                    Ok(self.single_char_token(TokenKind::Assign))
                }
            }

            '!' => {
                if self.peek_next() == '=' {
                    let tok = self.make_token(TokenKind::Neq, "!=");
                    self.advance();
                    self.advance();
                    Ok(tok)
                } else {
                    Ok(self.single_char_token(TokenKind::Not))
                }
            }

            '<' => {
                if self.peek_next() == '=' {
                    let tok = self.make_token(TokenKind::Lte, "<=");
                    self.advance();
                    self.advance();
                    Ok(tok)
                } else {
                    Ok(self.single_char_token(TokenKind::Lt))
                }
            }

            '>' => {
                if self.peek_next() == '=' {
                    let tok = self.make_token(TokenKind::Gte, ">=");
                    self.advance();
                    self.advance();
                    Ok(tok)
                } else {
                    Ok(self.single_char_token(TokenKind::Gt))
                }
            }

            '&' => {
                if self.peek_next() == '&' {
                    let tok = self.make_token(TokenKind::And, "&&");
                    self.advance();
                    self.advance();
                    Ok(tok)
                } else {
                    Err(self.error("unexpected character '&'"))
                }
            }

            '|' => {
                if self.peek_next() == '|' {
                    let tok = self.make_token(TokenKind::Or, "||");
                    self.advance();
                    self.advance();
                    Ok(tok)
                } else {
                    Err(self.error("unexpected character '|'"))
                }
            }

            '/' => {
                // Division (comments already handled by skip_comments).
                Ok(self.single_char_token(TokenKind::Slash))
            }

            _ => Err(self.error(&format!("unexpected character '{}'", ch))),
        }
    }

    // ── Token readers ──────────────────────────────────────────────────

    fn read_number(&mut self) -> Result<Token, LexerError> {
        let start_col = self.column;
        let mut num_str = String::new();

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            num_str.push(self.peek());
            self.advance();
        }

        let value: u64 = num_str
            .parse()
            .map_err(|_| self.error(&format!("invalid number literal: {}", num_str)))?;

        Ok(Token {
            kind: TokenKind::IntLiteral(value),
            lexeme: num_str,
            line: self.line,
            column: start_col,
        })
    }

    fn read_identifier(&mut self) -> Token {
        let start_col = self.column;
        let mut ident = String::new();

        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            ident.push(self.peek());
            self.advance();
        }

        let kind = match ident.as_str() {
            "contract" => TokenKind::Contract,
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "state" => TokenKind::State,
            "emit" => TokenKind::Emit,
            "require" => TokenKind::Require,
            "self" => TokenKind::Self_,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "u64" => TokenKind::U64,
            "bool" => TokenKind::Bool,
            "address" => TokenKind::Address,
            "string" => TokenKind::StringType,
            "map" => TokenKind::Map,
            _ => TokenKind::Identifier(ident.clone()),
        };

        Token {
            kind,
            lexeme: ident,
            line: self.line,
            column: start_col,
        }
    }

    fn read_string(&mut self) -> Result<Token, LexerError> {
        let start_col = self.column;
        self.advance(); // consume opening '"'

        let mut value = String::new();
        while !self.is_at_end() && self.peek() != '"' {
            if self.peek() == '\\' {
                self.advance();
                if self.is_at_end() {
                    return Err(self.error("unterminated string escape"));
                }
                match self.peek() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    '"' => value.push('"'),
                    '\\' => value.push('\\'),
                    c => value.push(c),
                }
            } else {
                value.push(self.peek());
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(self.error("unterminated string literal"));
        }

        self.advance(); // consume closing '"'

        Ok(Token {
            kind: TokenKind::StringLiteral(value.clone()),
            lexeme: format!("\"{}\"", value),
            line: self.line,
            column: start_col,
        })
    }

    // ── Helpers ────────────────────────────────────────────────────────

    fn peek(&self) -> char {
        self.source.get(self.pos).copied().unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.source.get(self.pos + 1).copied().unwrap_or('\0')
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.peek().is_whitespace() {
            self.advance();
        }
    }

    fn skip_comments(&mut self) {
        if !self.is_at_end() && self.peek() == '/' && self.peek_next() == '/' {
            // Single-line comment: skip until end of line.
            while !self.is_at_end() && self.peek() != '\n' {
                self.advance();
            }
        }
    }

    fn single_char_token(&mut self, kind: TokenKind) -> Token {
        let lexeme = self.peek().to_string();
        let tok = self.make_token(kind, &lexeme);
        self.advance();
        tok
    }

    fn make_token(&self, kind: TokenKind, lexeme: &str) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_string(),
            line: self.line,
            column: self.column,
        }
    }

    fn error(&self, msg: &str) -> LexerError {
        LexerError {
            message: msg.to_string(),
            line: self.line,
            column: self.column,
        }
    }
}

// ── Error ──────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
#[error("Lexer error at {line}:{column}: {message}")]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let src = "let x = 42;";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Assign);
        assert_eq!(tokens[3].kind, TokenKind::IntLiteral(42));
        assert_eq!(tokens[4].kind, TokenKind::Semicolon);
        assert_eq!(tokens[5].kind, TokenKind::Eof);
    }

    #[test]
    fn test_contract_declaration() {
        let src = "contract Token { }";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Contract);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("Token".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[3].kind, TokenKind::RightBrace);
    }

    #[test]
    fn test_function_signature() {
        let src = "fn transfer(to: address, amount: u64) -> bool {";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("transfer".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::LeftParen);
        assert_eq!(tokens[3].kind, TokenKind::Identifier("to".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::Colon);
        assert_eq!(tokens[5].kind, TokenKind::Address);
    }

    #[test]
    fn test_operators() {
        let src = "a + b == c && d >= e";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[1].kind, TokenKind::Plus);
        assert_eq!(tokens[3].kind, TokenKind::Eq);
        assert_eq!(tokens[5].kind, TokenKind::And);
        assert_eq!(tokens[7].kind, TokenKind::Gte);
    }

    #[test]
    fn test_string_literal() {
        let src = r#""hello world""#;
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(
            tokens[0].kind,
            TokenKind::StringLiteral("hello world".to_string())
        );
    }

    #[test]
    fn test_comments_skipped() {
        let src = "let x = 1; // this is a comment\nlet y = 2;";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        // Comments should be stripped — we should see let, x, =, 1, ;, let, y, =, 2, ;, EOF
        let idents: Vec<&TokenKind> = tokens.iter().map(|t| &t.kind).collect();
        assert!(idents.contains(&&TokenKind::Identifier("y".to_string())));
    }

    #[test]
    fn test_self_keyword() {
        let src = "self.balances";
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Self_);
        assert_eq!(tokens[1].kind, TokenKind::Dot);
    }

    #[test]
    fn test_full_contract() {
        let src = r#"
contract Token {
    state owner: address;
    state total: u64;

    fn init() {
        self.owner = caller();
        self.total = 1000000;
    }

    fn transfer(to: address, amount: u64) {
        let balance = self.total;
        require(balance >= amount, "Insufficient");
        self.total = balance - amount;
    }
}
        "#;
        let mut lexer = Lexer::new(src);
        let tokens = lexer.tokenize().unwrap();

        // Should tokenize without errors and end with EOF.
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
        assert!(tokens.len() > 30);
    }
}
