# RustrialScript

A minimal, efficient programming language interpreter designed specifically for RustrialOS.

## Overview

RustrialScript is a lightweight interpreted language that runs directly in your operating system without requiring a standard library. It features:

- **No std, only core + alloc**: Works in bare-metal OS environments
- **Stack-based VM**: Efficient bytecode execution
- **Small memory footprint**: Fixed-size stack (256 values), minimal heap usage
- **Simple syntax**: Easy to learn and use
- **OS integration**: Direct access to VGA buffer and system functions

## Language Features

### Data Types
- **Integers** (`i32`): Whole numbers
- **Booleans** (`true`/`false`): Logical values
- **Nil**: Represents no value

### Variables
```rust
let x = 42;
let y = x + 10;
x = x * 2;  // reassignment
```

### Arithmetic Operations
```rust
let sum = 10 + 5;        // Addition
let diff = 10 - 5;       // Subtraction
let prod = 10 * 5;       // Multiplication
let quot = 10 / 5;       // Division
let rem = 10 % 3;        // Modulo
let neg = -10;           // Negation
```

### Comparison Operations
```rust
let eq = (5 == 5);       // Equal
let neq = (5 != 3);      // Not equal
let lt = (3 < 5);        // Less than
let gt = (5 > 3);        // Greater than
let lte = (3 <= 5);      // Less than or equal
let gte = (5 >= 3);      // Greater than or equal
```

### Control Flow

#### If-Else Statements
```rust
if (x > 10) {
    print(1);
} else {
    print(0);
}
```

#### While Loops
```rust
let i = 0;
while (i < 10) {
    print(i);
    i = i + 1;
}
```

### Built-in Functions

#### `print(value)`
Prints a value to the screen
```rust
print(42);
print(x + y);
```

#### `clear()`
Clears the VGA text buffer
```rust
clear();
```

### Comments
```rust
// This is a single-line comment
let x = 5;  // Comments can be at the end of lines
```

## Example Programs

### Hello World (Numbers)
```rust
print(1337);
```

### Fibonacci Sequence
```rust
let a = 0;
let b = 1;
let n = 10;
let i = 0;

print(a);
print(b);

while (i < n) {
    let temp = a + b;
    print(temp);
    a = b;
    b = temp;
    i = i + 1;
}
```

### Factorial Calculator
```rust
let n = 5;
let result = 1;
let i = 1;

while (i <= n) {
    result = result * i;
    i = i + 1;
}

print(result);  // Prints 120
```

### Prime Number Checker
```rust
let num = 17;
let is_prime = 1;
let i = 2;

if (num < 2) {
    is_prime = 0;
}

while (i * i <= num) {
    if (num % i == 0) {
        is_prime = 0;
    }
    i = i + 1;
}

print(is_prime);  // 1 for prime, 0 for not prime
```

### Count to 100
```rust
clear();
let i = 1;
while (i <= 100) {
    print(i);
    i = i + 1;
}
```

## Architecture

### Components

1. **Lexer** (`lexer.rs`)
   - Tokenizes source code into tokens
   - Handles numbers, identifiers, keywords, and operators
   - Supports single-line comments (`//`)

2. **Parser** (`parser.rs`)
   - Parses tokens into bytecode instructions
   - Implements recursive descent parsing
   - Generates optimized bytecode

3. **Virtual Machine** (`vm.rs`)
   - Stack-based execution engine
   - 256-value stack for computations
   - HashMap for variable storage
   - Direct OS integration

4. **Value System** (`value.rs`)
   - Type-safe value representation
   - Integer and boolean operations
   - Nil value support

### Bytecode Instructions

The VM uses a compact bytecode format:

- **Stack Operations**: `Constant`, `LoadVar`, `StoreVar`, `Pop`
- **Arithmetic**: `Add`, `Subtract`, `Multiply`, `Divide`, `Modulo`, `Negate`
- **Comparison**: `Equal`, `NotEqual`, `Less`, `Greater`, `LessEqual`, `GreaterEqual`
- **Control Flow**: `Jump`, `JumpIfFalse`
- **Built-ins**: `Print`, `Clear`

## Usage in RustrialOS

### Basic Usage
```rust
use rustrial_os::rustrial_script;

let script = r#"
    let x = 42;
    print(x);
"#;

match rustrial_script::run(script) {
    Ok(_) => println!("Success!"),
    Err(e) => println!("Error: {}", e),
}
```

### Advanced Usage
```rust
use rustrial_os::rustrial_script::{VirtualMachine, lexer, parser};

// Compile once, run multiple times
let tokens = lexer::tokenize(source)?;
let bytecode = parser::parse(&tokens)?;

let mut vm = VirtualMachine::new();
vm.execute(&bytecode)?;
```

# RustrialScript Quick Reference

## Syntax at a Glance

### Variables
```rust
let x = 42;           // Declaration
x = 100;              // Reassignment
```

### Operators
```rust
// Arithmetic
+ - * / %             // Add, Sub, Mul, Div, Mod
-x                    // Negate

// Comparison
== != < > <= >=       // Equality, inequality, comparisons

// Precedence (high to low)
()                    // Parentheses
- (unary)             // Negation
* / %                 // Multiplication, division, modulo
+ -                   // Addition, subtraction
== != < > <= >=       // Comparisons
```

### Control Flow
```rust
// If-else
if (condition) {
    // code
} else {
    // code
}

// While loop
while (condition) {
    // code
}
```

### Built-in Functions
```rust
print(value);         // Print to screen
clear();              // Clear screen
```

### Comments
```rust
// Single-line comment
let x = 5;  // End-of-line comment
```

## Complete Examples

### Count 1-10
```rust
let i = 1;
while (i <= 10) {
    print(i);
    i = i + 1;
}
```

### Find Max
```rust
let a = 15;
let b = 23;
let max = a;
if (b > a) {
    max = b;
}
print(max);
```

### Factorial
```rust
let n = 5;
let result = 1;
let i = 1;
while (i <= n) {
    result = result * i;
    i = i + 1;
}
print(result);
```

### Fibonacci
```rust
let a = 0;
let b = 1;
let n = 10;
let i = 0;
print(a);
print(b);
while (i < n) {
    let temp = a + b;
    print(temp);
    a = b;
    b = temp;
    i = i + 1;
}
```

## Common Patterns

### Loop N times
```rust
let i = 0;
while (i < n) {
    // body
    i = i + 1;
}
```

### Conditional Assignment
```rust
let result = 0;
if (condition) {
    result = 1;
} else {
    result = 2;
}
```

### Accumulator Pattern
```rust
let sum = 0;
let i = 1;
while (i <= n) {
    sum = sum + i;
    i = i + 1;
}
```

### Boolean Flag
```rust
let found = 0;  // false
if (condition) {
    found = 1;  // true
}
```

## Limitations

- ❌ No strings (yet)
- ❌ No functions
- ❌ No arrays
- ❌ Only integers (i32)
- ❌ Only 256 stack values
- ✅ Variables unlimited (heap)
- ✅ Wrapping arithmetic
- ✅ Fast bytecode execution

## Tips

1. **Use parentheses** for complex expressions
2. **Initialize before use** - no implicit nil
3. **Check for zero** before division
4. **Use comments** to document logic
5. **Test incrementally** - build up complex programs

## Error Messages

| Error | Cause |
|-------|-------|
| "Stack overflow" | Too many values on stack |
| "Stack underflow" | Popped empty stack |
| "Undefined variable" | Variable not declared |
| "Division by zero" | Divided by 0 |
| "Expected integer" | Type mismatch |
| "Expected ';'" | Missing semicolon |
| "Expected ')'" | Missing closing paren |

## Performance Notes

- **Fast**: Direct bytecode execution
- **Compact**: ~8 bytes per instruction
- **Efficient**: Fixed-size stack
- **Optimal**: Compile once, run many times

## Memory Usage

| Component | Size |
|-----------|------|
| Stack | ~1 KB (256 values) |
| Variables | Dynamic (grows) |
| Bytecode | ~8 bytes/instruction |

---

## Performance Characteristics

- **Memory Usage**: 
  - Stack: ~1 KB (256 × 4 bytes per value)
  - Variables: Dynamic (BTreeMap, grows as needed)
  - Bytecode: ~8 bytes per instruction

- **Execution Speed**:
  - Direct bytecode interpretation
  - No AST traversal overhead
  - Wrapping arithmetic (no overflow checks)


## Future Enhancements

Potential features for future versions:
- String support with fixed-length string pool
- Functions and function calls
- Arrays with fixed-size allocation
- More built-in functions (colors, graphics, etc.)
- REPL (Read-Eval-Print Loop) mode
- Debugger and step-through execution

## License

Part of RustrialOS - A Rust-based operating system.

## Contributing

To add new features:
1. Update the lexer to recognize new syntax
2. Add parsing logic in the parser
3. Implement new bytecode operations in the VM
4. Update this README with examples

---

**Created for RustrialOS by the OS development team**
