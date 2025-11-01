# RustrialScript Integration Guide

This document explains how RustrialScript is integrated into RustrialOS and how to extend it.

## Architecture Overview

```
┌─────────────────────────────────────────────┐
│           RustrialOS (no_std)               │
├─────────────────────────────────────────────┤
│  Kernel │ VGA Buffer │ Allocator │ Tasks   │
├─────────────────────────────────────────────┤
│           RustrialScript Interpreter        │
│  ┌─────────┬─────────┬──────┬──────────┐  │
│  │ Lexer   │ Parser  │ VM   │ Values   │  │
│  └─────────┴─────────┴──────┴──────────┘  │
└─────────────────────────────────────────────┘
```

## Module Structure

```
src/rustrial_script/
├── mod.rs           - Main module interface
├── lexer.rs         - Tokenization
├── parser.rs        - Bytecode compilation
├── vm.rs            - Virtual machine execution
├── value.rs         - Value types
├── tests.rs         - Unit tests
├── README.md        - Language documentation
└── examples/        - Example scripts
    ├── fibonacci.rscript
    ├── factorial.rscript
    ├── prime_checker.rscript
    ├── sum_of_squares.rscript
    ├── collatz.rscript
    └── gcd.rscript
```

## Key Design Decisions

### 1. No Standard Library (`#![no_std]`)
All modules use only `core` and `alloc`:
- `core` for basic types and traits
- `alloc` for `Vec`, `String`, and `BTreeMap`
- No file I/O, no `println!` (uses OS's `print!` macro)

### 2. Stack-Based VM
The VM uses a fixed-size stack for efficiency:
```rust
const STACK_SIZE: usize = 256;
stack: [Value; STACK_SIZE],
```

This avoids heap allocation during execution.

### 3. Bytecode Compilation
Source code is compiled to bytecode before execution:
```
Source → Tokens → Bytecode → Execution
```

This allows for:
- Faster execution (no re-parsing)
- Early error detection
- Potential optimization passes

### 4. Direct OS Integration
The VM directly calls OS functions:
```rust
OpCode::Print => {
    let value = self.pop()?;
    println!("{}", value);  // Uses OS println! macro
}

OpCode::Clear => {
    use crate::vga_buffer::WRITER;
    WRITER.lock().clear_screen();
}
```

## Adding New Features

### Adding a New Operator

1. **Update the Lexer** (`lexer.rs`):
```rust
// Add token type
pub enum Token {
    // ...existing tokens...
    Power,  // New: ** operator
}

// Update tokenize function
'*' => {
    chars.next();
    if chars.peek() == Some(&'*') {
        chars.next();
        tokens.push(Token::Power);
    } else {
        tokens.push(Token::Star);
    }
}
```

2. **Update the Parser** (`parser.rs`):
```rust
// Add opcode
pub enum OpCode {
    // ...existing opcodes...
    Power,  // New: exponentiation
}

// Update parsing (in parse_factor or similar)
while matches!(self.peek(), Token::Star | Token::Power) {
    let op = self.advance().clone();
    self.parse_unary()?;
    
    match op {
        Token::Star => self.emit(OpCode::Multiply),
        Token::Power => self.emit(OpCode::Power),
        _ => unreachable!(),
    }
}
```

3. **Update the VM** (`vm.rs`):
```rust
OpCode::Power => {
    let exp = self.pop()?.as_int()?;
    let base = self.pop()?.as_int()?;
    
    let mut result = 1;
    let mut e = exp;
    while e > 0 {
        result = result.wrapping_mul(base);
        e -= 1;
    }
    
    self.push(Value::Int(result))?;
}
```

### Adding a New Built-in Function

1. **Add keyword to Lexer**:
```rust
pub enum Token {
    // ...
    GetTicks,  // New: get system ticks
}

let token = match ident.as_str() {
    // ...existing keywords...
    "getticks" => Token::GetTicks,
    _ => Token::Identifier(ident),
};
```

2. **Add parsing**:
```rust
fn parse_statement(&mut self) -> Result<(), &'static str> {
    match self.peek() {
        // ...existing cases...
        Token::GetTicks => self.parse_getticks(),
        // ...
    }
}

fn parse_getticks(&mut self) -> Result<(), &'static str> {
    self.advance();
    self.consume(Token::LeftParen, "Expected '('")?;
    self.consume(Token::RightParen, "Expected ')'")?;
    self.consume(Token::Semicolon, "Expected ';'")?;
    self.emit(OpCode::GetTicks);
    Ok(())
}
```

3. **Implement in VM**:
```rust
pub enum OpCode {
    // ...
    GetTicks,
}

// In execute():
OpCode::GetTicks => {
    // Get system time/ticks from OS
    use x86_64::instructions::interrupts;
    let ticks = /* get from OS timer */;
    self.push(Value::Int(ticks as i32))?;
}
```

### Adding a New Value Type

To add strings (example):

1. **Update Value enum** (`value.rs`):
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i32),
    Bool(bool),
    String(alloc::string::String),  // New
    Nil,
}

impl Value {
    pub fn as_string(&self) -> Result<&str, &'static str> {
        match self {
            Value::String(s) => Ok(s.as_str()),
            _ => Err("Expected string"),
        }
    }
}
```

2. **Update Lexer** for string literals:
```rust
'"' => {
    chars.next();
    let mut string = String::new();
    while let Some(&ch) = chars.peek() {
        if ch == '"' {
            chars.next();
            break;
        }
        string.push(ch);
        chars.next();
    }
    tokens.push(Token::StringLiteral(string));
}
```

3. **Update Parser and VM** to handle string operations.

## Memory Management

### Stack Usage
- Fixed 256-value stack: ~1-2 KB
- No dynamic allocation during execution

### Heap Usage
- Variable storage: BTreeMap (grows as needed)
- Bytecode: Vec of opcodes
- Token storage: Vec of tokens

### Recommendations
- Keep scripts small (< 100 lines)
- Limit variable count (< 50 variables)
- Avoid deep recursion (no recursion support yet)

## Performance Tips

### Optimization Opportunities

1. **Constant Folding**
   - Evaluate constant expressions at compile time
   - Example: `5 + 3` → `Constant(8)`

2. **Variable Inlining**
   - Replace single-use variables with their values
   - Requires dataflow analysis

3. **Jump Threading**
   - Optimize redundant jumps
   - Example: `Jump(x)` where x is another Jump

4. **Register Allocation**
   - Keep frequently-used variables in "registers" (array indices)
   - Reduces HashMap lookups

## Testing

Run tests with:
```bash
cargo test --lib rustrial_script
```

Key test categories:
- Lexer: Token generation
- Parser: Bytecode generation
- VM: Execution correctness
- Integration: End-to-end scripts

## Debugging

### Enable Debug Logging

Modify `vm.rs` to print bytecode:
```rust
pub fn execute(&mut self, bytecode: &[OpCode]) -> Result<(), &'static str> {
    #[cfg(debug_assertions)]
    {
        println!("=== Bytecode ===");
        for (i, op) in bytecode.iter().enumerate() {
            println!("{}: {:?}", i, op);
        }
    }
    
    // ...rest of execution...
}
```

### Stack Inspection

Add to VM:
```rust
fn print_stack(&self) {
    println!("Stack (top={}): {:?}", self.stack_top, 
             &self.stack[..self.stack_top]);
}
```

## Future Enhancements

### Planned Features
1. **Functions**: User-defined functions with parameters
2. **Arrays**: Fixed-size array support
3. **Strings**: Full string manipulation
4. **Modules**: Code organization across files
5. **FFI**: Call OS functions directly

### Long-term Goals
1. **JIT Compilation**: Compile hot paths to native code
2. **REPL**: Interactive interpreter
3. **Debugger**: Step-through execution
4. **IDE Support**: Syntax highlighting, autocomplete

## Troubleshooting

### Common Issues

**Stack Overflow**
```
Error: Stack overflow
```
Solution: Reduce recursion depth or increase `STACK_SIZE`

**Undefined Variable**
```
Error: Undefined variable
```
Solution: Check variable names, ensure declaration before use

**Division by Zero**
```
Error: Division by zero
```
Solution: Add runtime check before division

**Parse Error**
```
Error: Expected ';'
```
Solution: Check syntax, ensure all statements end with semicolon

## Contributing

When adding features:
1. Update lexer, parser, VM as needed
2. Add tests in `tests.rs`
3. Update documentation in `README.md`
4. Add example scripts if applicable
5. Test in real OS environment

## Contact

For questions or issues, please refer to the main RustrialOS documentation.

---

**Last Updated**: November 1, 2025
**Version**: 1.0.0
