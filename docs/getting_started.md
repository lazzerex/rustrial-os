# Getting Started with RustrialScript

Welcome! This guide will help you start using RustrialScript in your RustrialOS.

## Quick Start

### 1. Build and Run
```bash
cd rustrial_os
cargo build
cargo run
```

You should see the Fibonacci demo execute automatically!

### 2. Your First Script

Create a simple script:

```rust
// In src/main.rs, replace the demo script with:

let script = r#"
    let x = 10;
    let y = 20;
    let sum = x + y;
    print(sum);
"#;

match rustrial_os::rustrial_script::run(script) {
    Ok(_) => println!("Success!"),
    Err(e) => println!("Error: {}", e),
}
```

### 3. Learn the Syntax

#### Variables
```rust
let age = 25;           // Create variable
age = 26;               // Update variable
```

#### Math
```rust
let a = 10 + 5;         // 15
let b = 10 - 3;         // 7
let c = 4 * 5;          // 20
let d = 20 / 4;         // 5
let e = 10 % 3;         // 1 (remainder)
let f = -10;            // -10 (negative)
```

#### Comparisons
```rust
let eq = (5 == 5);      // true
let ne = (5 != 3);      // true
let lt = (3 < 5);       // true
let gt = (5 > 3);       // true
let le = (5 <= 5);      // true
let ge = (5 >= 5);      // true
```

#### If-Else
```rust
let x = 10;
if (x > 5) {
    print(1);           // This runs
} else {
    print(0);
}
```

#### Loops
```rust
let i = 1;
while (i <= 5) {
    print(i);           // Prints 1, 2, 3, 4, 5
    i = i + 1;
}
```

#### Built-ins
```rust
print(42);              // Print value
clear();                // Clear screen
```

## Example Programs

### Count to 10
```rust
let i = 1;
while (i <= 10) {
    print(i);
    i = i + 1;
}
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

print(result);  // Prints 120
```

### Sum of Numbers
```rust
let n = 10;
let sum = 0;
let i = 1;

while (i <= n) {
    sum = sum + i;
    i = i + 1;
}

print(sum);  // Prints 55
```

### Even or Odd
```rust
let num = 42;
if (num % 2 == 0) {
    print(1);   // Even
} else {
    print(0);   // Odd
}
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

## Running Scripts in Your OS

### Method 1: Hardcoded Script
```rust
// In main.rs
let script = r#"
    // Your script here
    let x = 42;
    print(x);
"#;

rustrial_os::rustrial_script::run(script).unwrap();
```

### Method 2: Multiple Scripts
```rust
// Run different scripts based on conditions
let script1 = "let x = 10; print(x);";
let script2 = "let y = 20; print(y);";

if some_condition {
    rustrial_os::rustrial_script::run(script1)?;
} else {
    rustrial_os::rustrial_script::run(script2)?;
}
```

### Method 3: Error Handling
```rust
let script = "let x = 10 / 0;";  // Will error!

match rustrial_os::rustrial_script::run(script) {
    Ok(_) => println!("Script succeeded"),
    Err(e) => println!("Script failed: {}", e),
}
```

## Common Patterns

### Loop Counter
```rust
let i = 0;
while (i < 10) {
    // Do something
    i = i + 1;
}
```

### Accumulator
```rust
let sum = 0;
let i = 1;
while (i <= 10) {
    sum = sum + i;
    i = i + 1;
}
```

### Flag/Boolean
```rust
let found = 0;  // false
// ... some logic ...
if (condition) {
    found = 1;  // true
}
```

### Min/Max
```rust
let a = 15;
let b = 23;
let max = a;
if (b > a) {
    max = b;
}
print(max);
```

## Tips and Tricks

### 1. Always Initialize Variables
```rust
// Good
let x = 0;
x = 42;

// Bad - will error!
print(x);  // x not defined
```

### 2. Check Before Division
```rust
let divisor = 0;
if (divisor != 0) {
    let result = 100 / divisor;
    print(result);
}
```

### 3. Use Parentheses for Clarity
```rust
// Clear
let result = (a + b) * (c - d);

// Less clear
let result = a + b * c - d;
```

### 4. Comment Your Code
```rust
// Calculate sum of squares
let sum = 0;
let i = 1;
while (i <= 10) {
    sum = sum + (i * i);  // Add square of i
    i = i + 1;
}
```

### 5. Test Small Pieces First
```rust
// Test simple operations first
let x = 5;
print(x);  // Make sure print works

// Then build up
let y = x + 3;
print(y);  // Test addition
```

## Debugging

### Print Variables
```rust
let x = 42;
print(x);  // Check value
```

### Print at Key Points
```rust
let i = 0;
while (i < 5) {
    print(i);  // See loop progress
    i = i + 1;
}
```

### Use Clear Screen
```rust
clear();        // Start fresh
// Your code here
```

### Simplify Complex Logic
```rust
// Instead of:
if (a > b) {
    if (c < d) {
        // complex logic
    }
}

// Try:
let cond1 = (a > b);
let cond2 = (c < d);
print(cond1);  // Debug
print(cond2);  // Debug
```

## Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| "Stack overflow" | Too many values | Simplify expressions |
| "Undefined variable" | Typo or not declared | Check spelling, add `let` |
| "Division by zero" | Divided by 0 | Check divisor before dividing |
| "Expected ';'" | Missing semicolon | Add `;` at end of statement |
| "Expected ')'" | Missing paren | Match all `(` with `)` |

## Next Steps

1. **Read the full docs**: See `src/rustrial_script/README.md`
2. **Try examples**: Check `src/rustrial_script/examples/`
3. **Experiment**: Modify the demo script in `main.rs`
4. **Extend**: Read `INTEGRATION.md` to add features

## Resources

- **Full Language Reference**: `src/rustrial_script/README.md`
- **Quick Reference**: `src/rustrial_script/QUICKREF.md`
- **Integration Guide**: `src/rustrial_script/INTEGRATION.md`
- **Architecture**: `ARCHITECTURE.md`
- **Examples**: `src/rustrial_script/examples/`

## Need Help?

Common questions:

**Q: Can I use strings?**
A: Not yet, only integers and booleans. Strings are planned for future versions.

**Q: Can I define functions?**
A: Not yet, but it's a planned feature.

**Q: How do I read keyboard input?**
A: This requires OS integration. You'd need to add a built-in function.

**Q: Can I call OS functions?**
A: Not directly, but you can add built-in functions that call OS code.

**Q: Is it fast?**
A: Yes! It uses bytecode compilation and direct execution.

## Have Fun!

RustrialScript is designed to be simple and fun to use. Start with small programs and build up to more complex ones. Happy coding!

---

**Welcome to RustrialScript!** 🦀
