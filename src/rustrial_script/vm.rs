//! Virtual Machine for executing RustrialScript bytecode

use alloc::collections::BTreeMap;
use alloc::string::String;
use crate::rustrial_script::parser::OpCode;
use crate::rustrial_script::value::Value;
use crate::println;

const STACK_SIZE: usize = 256;

pub struct VirtualMachine {
    stack: [Value; STACK_SIZE],
    stack_top: usize,
    variables: BTreeMap<String, Value>,
    ip: usize, // instruction pointer
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: [Value::Nil; STACK_SIZE],
            stack_top: 0,
            variables: BTreeMap::new(),
            ip: 0,
        }
    }
    
    fn push(&mut self, value: Value) -> Result<(), &'static str> {
        if self.stack_top >= STACK_SIZE {
            return Err("Stack overflow");
        }
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
        Ok(())
    }
    
    fn pop(&mut self) -> Result<Value, &'static str> {
        if self.stack_top == 0 {
            return Err("Stack underflow");
        }
        self.stack_top -= 1;
        Ok(self.stack[self.stack_top].clone())
    }
    
    fn peek(&self) -> Result<&Value, &'static str> {
        if self.stack_top == 0 {
            return Err("Stack is empty");
        }
        Ok(&self.stack[self.stack_top - 1])
    }
    
    pub fn execute(&mut self, bytecode: &[OpCode]) -> Result<(), &'static str> {
        self.ip = 0;
        
        while self.ip < bytecode.len() {
            let op = &bytecode[self.ip];
            self.ip += 1;
            
            match op {
                OpCode::Constant(n) => {
                    self.push(Value::Int(*n))?;
                }
                OpCode::LoadVar(name) => {
                    let value = self.variables.get(name)
                        .ok_or("Undefined variable")?
                        .clone();
                    self.push(value)?;
                }
                OpCode::StoreVar(name) => {
                    let value = self.pop()?;
                    self.variables.insert(name.clone(), value);
                }
                OpCode::Add => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Int(a.wrapping_add(b)))?;
                }
                OpCode::Subtract => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Int(a.wrapping_sub(b)))?;
                }
                OpCode::Multiply => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Int(a.wrapping_mul(b)))?;
                }
                OpCode::Divide => {
                    let b = self.pop()?.as_int()?;
                    if b == 0 {
                        return Err("Division by zero");
                    }
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Int(a / b))?;
                }
                OpCode::Modulo => {
                    let b = self.pop()?.as_int()?;
                    if b == 0 {
                        return Err("Modulo by zero");
                    }
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Int(a % b))?;
                }
                OpCode::Negate => {
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Int(-a))?;
                }
                OpCode::Equal => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a == b))?;
                }
                OpCode::NotEqual => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    self.push(Value::Bool(a != b))?;
                }
                OpCode::Less => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Bool(a < b))?;
                }
                OpCode::Greater => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Bool(a > b))?;
                }
                OpCode::LessEqual => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Bool(a <= b))?;
                }
                OpCode::GreaterEqual => {
                    let b = self.pop()?.as_int()?;
                    let a = self.pop()?.as_int()?;
                    self.push(Value::Bool(a >= b))?;
                }
                OpCode::Jump(target) => {
                    self.ip = *target;
                }
                OpCode::JumpIfFalse(target) => {
                    let condition = self.peek()?.is_truthy();
                    if !condition {
                        self.ip = *target;
                    }
                    self.pop()?; // Pop the condition value
                }
                OpCode::Print => {
                    let value = self.pop()?;
                    println!("{}", value);
                }
                OpCode::Clear => {
                    // Clear the screen
                    use crate::vga_buffer::WRITER;
                    WRITER.lock().clear_screen();
                }
                OpCode::Pop => {
                    self.pop()?;
                }
            }
        }
        
        Ok(())
    }
}
