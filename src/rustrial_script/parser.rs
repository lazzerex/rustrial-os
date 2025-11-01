//! Parser and bytecode compiler for RustrialScript

use alloc::vec::Vec;
use alloc::string::String;
use crate::rustrial_script::lexer::Token;

#[derive(Debug, Clone)]
pub enum OpCode {
    // Stack operations
    Constant(i32),
    LoadVar(String),
    StoreVar(String),
    
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Negate,
    
    // Comparison
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    
    // Control flow
    Jump(usize),           // Unconditional jump
    JumpIfFalse(usize),    // Jump if top of stack is false
    
    // Built-ins
    Print,
    Clear,
    
    // Stack management
    Pop,
}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    bytecode: Vec<OpCode>,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            bytecode: Vec::new(),
        }
    }
    
    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }
    
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        &self.tokens[self.current - 1]
    }
    
    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            return false;
        }
        core::mem::discriminant(self.peek()) == core::mem::discriminant(token)
    }
    
    fn consume(&mut self, expected: Token, msg: &'static str) -> Result<(), &'static str> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(msg)
        }
    }
    
    fn emit(&mut self, op: OpCode) {
        self.bytecode.push(op);
    }
    
    fn parse_program(&mut self) -> Result<(), &'static str> {
        while !self.is_at_end() {
            self.parse_statement()?;
        }
        Ok(())
    }
    
    fn parse_statement(&mut self) -> Result<(), &'static str> {
        match self.peek() {
            Token::Let => self.parse_let(),
            Token::If => self.parse_if(),
            Token::While => self.parse_while(),
            Token::Print => self.parse_print(),
            Token::Clear => self.parse_clear(),
            Token::LeftBrace => self.parse_block(),
            Token::Identifier(_) => self.parse_assignment_or_expr(),
            _ => {
                self.parse_expression()?;
                self.consume(Token::Semicolon, "Expected ';'")?;
                self.emit(OpCode::Pop);
                Ok(())
            }
        }
    }
    
    fn parse_let(&mut self) -> Result<(), &'static str> {
        self.advance(); // consume 'let'
        
        let name = match self.advance() {
            Token::Identifier(n) => n.clone(),
            _ => return Err("Expected identifier after 'let'"),
        };
        
        self.consume(Token::Equal, "Expected '=' after variable name")?;
        self.parse_expression()?;
        self.consume(Token::Semicolon, "Expected ';'")?;
        
        self.emit(OpCode::StoreVar(name));
        Ok(())
    }
    
    fn parse_assignment_or_expr(&mut self) -> Result<(), &'static str> {
        let name = match self.peek() {
            Token::Identifier(n) => n.clone(),
            _ => return Err("Expected identifier"),
        };
        self.advance();
        
        if self.check(&Token::Equal) {
            self.advance(); // consume '='
            self.parse_expression()?;
            self.consume(Token::Semicolon, "Expected ';'")?;
            self.emit(OpCode::StoreVar(name));
            Ok(())
        } else {
            // It's an expression starting with identifier
            self.current -= 1; // backtrack
            self.parse_expression()?;
            self.consume(Token::Semicolon, "Expected ';'")?;
            self.emit(OpCode::Pop);
            Ok(())
        }
    }
    
    fn parse_if(&mut self) -> Result<(), &'static str> {
        self.advance(); // consume 'if'
        
        self.consume(Token::LeftParen, "Expected '(' after 'if'")?;
        self.parse_expression()?;
        self.consume(Token::RightParen, "Expected ')' after condition")?;
        
        // Placeholder for jump instruction
        let jump_if_false_idx = self.bytecode.len();
        self.emit(OpCode::JumpIfFalse(0));
        
        self.parse_statement()?;
        
        if self.check(&Token::Else) {
            self.advance(); // consume 'else'
            
            // Jump over else branch
            let jump_idx = self.bytecode.len();
            self.emit(OpCode::Jump(0));
            
            // Patch the jump_if_false to here
            let else_start = self.bytecode.len();
            self.bytecode[jump_if_false_idx] = OpCode::JumpIfFalse(else_start);
            
            self.parse_statement()?;
            
            // Patch the jump to after else
            let after_else = self.bytecode.len();
            self.bytecode[jump_idx] = OpCode::Jump(after_else);
        } else {
            // No else branch, patch jump to here
            let after_if = self.bytecode.len();
            self.bytecode[jump_if_false_idx] = OpCode::JumpIfFalse(after_if);
        }
        
        Ok(())
    }
    
    fn parse_while(&mut self) -> Result<(), &'static str> {
        self.advance(); // consume 'while'
        
        let loop_start = self.bytecode.len();
        
        self.consume(Token::LeftParen, "Expected '(' after 'while'")?;
        self.parse_expression()?;
        self.consume(Token::RightParen, "Expected ')' after condition")?;
        
        let jump_if_false_idx = self.bytecode.len();
        self.emit(OpCode::JumpIfFalse(0));
        
        self.parse_statement()?;
        
        self.emit(OpCode::Jump(loop_start));
        
        let after_loop = self.bytecode.len();
        self.bytecode[jump_if_false_idx] = OpCode::JumpIfFalse(after_loop);
        
        Ok(())
    }
    
    fn parse_print(&mut self) -> Result<(), &'static str> {
        self.advance(); // consume 'print'
        
        self.consume(Token::LeftParen, "Expected '(' after 'print'")?;
        self.parse_expression()?;
        self.consume(Token::RightParen, "Expected ')' after expression")?;
        self.consume(Token::Semicolon, "Expected ';'")?;
        
        self.emit(OpCode::Print);
        Ok(())
    }
    
    fn parse_clear(&mut self) -> Result<(), &'static str> {
        self.advance(); // consume 'clear'
        
        self.consume(Token::LeftParen, "Expected '(' after 'clear'")?;
        self.consume(Token::RightParen, "Expected ')'")?;
        self.consume(Token::Semicolon, "Expected ';'")?;
        
        self.emit(OpCode::Clear);
        Ok(())
    }
    
    fn parse_block(&mut self) -> Result<(), &'static str> {
        self.advance(); // consume '{'
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            self.parse_statement()?;
        }
        
        self.consume(Token::RightBrace, "Expected '}'")?;
        Ok(())
    }
    
    fn parse_expression(&mut self) -> Result<(), &'static str> {
        self.parse_comparison()
    }
    
    fn parse_comparison(&mut self) -> Result<(), &'static str> {
        self.parse_term()?;
        
        while matches!(self.peek(), 
            Token::EqualEqual | Token::BangEqual | 
            Token::Less | Token::Greater | 
            Token::LessEqual | Token::GreaterEqual) {
            
            let op = self.advance().clone();
            self.parse_term()?;
            
            match op {
                Token::EqualEqual => self.emit(OpCode::Equal),
                Token::BangEqual => self.emit(OpCode::NotEqual),
                Token::Less => self.emit(OpCode::Less),
                Token::Greater => self.emit(OpCode::Greater),
                Token::LessEqual => self.emit(OpCode::LessEqual),
                Token::GreaterEqual => self.emit(OpCode::GreaterEqual),
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    fn parse_term(&mut self) -> Result<(), &'static str> {
        self.parse_factor()?;
        
        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let op = self.advance().clone();
            self.parse_factor()?;
            
            match op {
                Token::Plus => self.emit(OpCode::Add),
                Token::Minus => self.emit(OpCode::Subtract),
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    fn parse_factor(&mut self) -> Result<(), &'static str> {
        self.parse_unary()?;
        
        while matches!(self.peek(), Token::Star | Token::Slash | Token::Percent) {
            let op = self.advance().clone();
            self.parse_unary()?;
            
            match op {
                Token::Star => self.emit(OpCode::Multiply),
                Token::Slash => self.emit(OpCode::Divide),
                Token::Percent => self.emit(OpCode::Modulo),
                _ => unreachable!(),
            }
        }
        
        Ok(())
    }
    
    fn parse_unary(&mut self) -> Result<(), &'static str> {
        if self.check(&Token::Minus) {
            self.advance();
            self.parse_unary()?;
            self.emit(OpCode::Negate);
            Ok(())
        } else {
            self.parse_primary()
        }
    }
    
    fn parse_primary(&mut self) -> Result<(), &'static str> {
        match self.advance().clone() {
            Token::Number(n) => {
                self.emit(OpCode::Constant(n));
                Ok(())
            }
            Token::Identifier(name) => {
                self.emit(OpCode::LoadVar(name));
                Ok(())
            }
            Token::LeftParen => {
                self.parse_expression()?;
                self.consume(Token::RightParen, "Expected ')' after expression")?;
                Ok(())
            }
            _ => Err("Unexpected token in expression"),
        }
    }
}

pub fn parse(tokens: &[Token]) -> Result<Vec<OpCode>, &'static str> {
    let mut parser = Parser::new(tokens.to_vec());
    parser.parse_program()?;
    Ok(parser.bytecode)
}
