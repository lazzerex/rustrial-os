//! RustrialScript - A minimal interpreter for RustrialOS
//! 
//! Features:
//! - Variables (integers and strings)
//! - Arithmetic operations (+, -, *, /, %)
//! - Comparisons (==, !=, <, >, <=, >=)
//! - Control flow (if/else, while)
//! - Built-in functions (print, clear, color)
//! - No heap allocation for execution (only for storage)

pub mod lexer;
pub mod parser;
pub mod vm;
pub mod value;


pub use vm::VirtualMachine;
pub use value::Value;

/// Run a RustrialScript program
pub fn run(source: &str) -> Result<(), &'static str> {
    let tokens = lexer::tokenize(source)?;
    let bytecode = parser::parse(&tokens)?;
    let mut vm = VirtualMachine::new();
    vm.execute(&bytecode)?;
    Ok(())
}
