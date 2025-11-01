//! Lexer for RustrialScript

use alloc::vec::Vec;
use alloc::string::String;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(i32),
    Identifier(String),
    
    // Keywords
    Let,
    If,
    Else,
    While,
    Print,
    Clear,
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    BangEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    
    Eof,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, &'static str> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    
    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\r' | '\n' => {
                chars.next();
            }
            '0'..='9' => {
                let mut num = 0i32;
                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        num = num.wrapping_mul(10).wrapping_add((ch as i32) - ('0' as i32));
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Number(num));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        ident.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }
                
                let token = match ident.as_str() {
                    "let" => Token::Let,
                    "if" => Token::If,
                    "else" => Token::Else,
                    "while" => Token::While,
                    "print" => Token::Print,
                    "clear" => Token::Clear,
                    _ => Token::Identifier(ident),
                };
                tokens.push(token);
            }
            '+' => {
                chars.next();
                tokens.push(Token::Plus);
            }
            '-' => {
                chars.next();
                tokens.push(Token::Minus);
            }
            '*' => {
                chars.next();
                tokens.push(Token::Star);
            }
            '/' => {
                chars.next();
                if chars.peek() == Some(&'/') {
                    // Comment - skip to end of line
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch == '\n' {
                            break;
                        }
                    }
                } else {
                    tokens.push(Token::Slash);
                }
            }
            '%' => {
                chars.next();
                tokens.push(Token::Percent);
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::EqualEqual);
                } else {
                    tokens.push(Token::Equal);
                }
            }
            '!' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::BangEqual);
                } else {
                    return Err("Unexpected character '!'");
                }
            }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::LessEqual);
                } else {
                    tokens.push(Token::Less);
                }
            }
            '>' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    chars.next();
                    tokens.push(Token::GreaterEqual);
                } else {
                    tokens.push(Token::Greater);
                }
            }
            '(' => {
                chars.next();
                tokens.push(Token::LeftParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RightParen);
            }
            '{' => {
                chars.next();
                tokens.push(Token::LeftBrace);
            }
            '}' => {
                chars.next();
                tokens.push(Token::RightBrace);
            }
            ';' => {
                chars.next();
                tokens.push(Token::Semicolon);
            }
            _ => {
                return Err("Unexpected character");
            }
        }
    }
    
    tokens.push(Token::Eof);
    Ok(tokens)
}
