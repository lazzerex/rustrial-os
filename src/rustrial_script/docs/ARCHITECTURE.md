# RustrialScript Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        RustrialOS Kernel                         │
│  ┌───────────┐  ┌────────────┐  ┌──────────┐  ┌──────────┐    │
│  │    GDT    │  │ Interrupts │  │  Memory  │  │  Tasks   │    │
│  └───────────┘  └────────────┘  └──────────┘  └──────────┘    │
│  ┌───────────┐  ┌────────────┐  ┌──────────┐                  │
│  │VGA Buffer │  │  Allocator │  │  Serial  │                  │
│  └─────┬─────┘  └──────┬─────┘  └──────────┘                  │
└────────┼────────────────┼────────────────────────────────────────┘
         │                │
         │                │
┌────────┼────────────────┼────────────────────────────────────────┐
│        │                │     RustrialScript Interpreter         │
│        │                │                                        │
│  ┌─────▼─────────────────▼─────────────────────────────┐        │
│  │              Virtual Machine (vm.rs)                 │        │
│  │  ┌────────────────────────────────────────────┐     │        │
│  │  │    Stack [256 Values]                      │     │        │
│  │  │  ┌──────┬──────┬──────┬──────┬──────┐     │     │        │
│  │  │  │ Val  │ Val  │ Val  │ ...  │ Top  │     │     │        │
│  │  │  └──────┴──────┴──────┴──────┴──────┘     │     │        │
│  │  └────────────────────────────────────────────┘     │        │
│  │  ┌────────────────────────────────────────────┐     │        │
│  │  │  Variables (BTreeMap<String, Value>)       │     │        │
│  │  │  { "x": Int(42), "y": Int(10), ... }       │     │        │
│  │  └────────────────────────────────────────────┘     │        │
│  │  ┌────────────────────────────────────────────┐     │        │
│  │  │  Instruction Pointer (ip)                  │     │        │
│  │  └────────────────────────────────────────────┘     │        │
│  └──────────────────────────────────────────────────────┘        │
│                            ▲                                     │
│                            │                                     │
│  ┌─────────────────────────┴──────────────────────────┐         │
│  │           Bytecode Instructions (OpCode)            │         │
│  │  ┌────────────────────────────────────────────┐    │         │
│  │  │ Constant(42)                               │    │         │
│  │  │ StoreVar("x")                              │    │         │
│  │  │ LoadVar("x")                               │    │         │
│  │  │ Constant(10)                               │    │         │
│  │  │ Add                                        │    │         │
│  │  │ Print                                      │    │         │
│  │  └────────────────────────────────────────────┘    │         │
│  └─────────────────────────────────────────────────────┘         │
│                            ▲                                     │
│                            │                                     │
│  ┌─────────────────────────┴──────────────────────────┐         │
│  │              Parser (parser.rs)                     │         │
│  │  - Recursive descent parsing                       │         │
│  │  - Compiles tokens to bytecode                     │         │
│  │  - Handles control flow (if/else, while)           │         │
│  │  - Generates jump instructions                     │         │
│  └─────────────────────────────────────────────────────┘         │
│                            ▲                                     │
│                            │                                     │
│  ┌─────────────────────────┴──────────────────────────┐         │
│  │                Tokens (Token)                       │         │
│  │  ┌────────────────────────────────────────────┐    │         │
│  │  │ Let                                        │    │         │
│  │  │ Identifier("x")                            │    │         │
│  │  │ Equal                                      │    │         │
│  │  │ Number(42)                                 │    │         │
│  │  │ Semicolon                                  │    │         │
│  │  └────────────────────────────────────────────┘    │         │
│  └─────────────────────────────────────────────────────┘         │
│                            ▲                                     │
│                            │                                     │
│  ┌─────────────────────────┴──────────────────────────┐         │
│  │              Lexer (lexer.rs)                       │         │
│  │  - Tokenizes source code                           │         │
│  │  - Handles numbers, identifiers, keywords          │         │
│  │  - Supports comments                               │         │
│  └─────────────────────────────────────────────────────┘         │
│                            ▲                                     │
│                            │                                     │
│  ┌─────────────────────────┴──────────────────────────┐         │
│  │              Source Code (&str)                     │         │
│  │  ┌────────────────────────────────────────────┐    │         │
│  │  │ let x = 42;                                │    │         │
│  │  │ let y = x + 10;                            │    │         │
│  │  │ print(y);                                  │    │         │
│  │  └────────────────────────────────────────────┘    │         │
│  └─────────────────────────────────────────────────────┘         │
└───────────────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────────────┐
│                        Value Types                                 │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐      │
│  │   Int(i32)     │  │   Bool(bool)   │  │     Nil        │      │
│  │   Example:     │  │   Example:     │  │   Example:     │      │
│  │   Int(42)      │  │   Bool(true)   │  │   Nil          │      │
│  └────────────────┘  └────────────────┘  └────────────────┘      │
└───────────────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────────────┐
│                      Execution Flow                                │
│                                                                    │
│  User Code → Lexer → Tokens → Parser → Bytecode → VM → Output    │
│                                                       │            │
│                                                       ↓            │
│                                             ┌──────────────────┐  │
│                                             │   VGA Buffer     │  │
│                                             │   (Screen)       │  │
│                                             └──────────────────┘  │
└───────────────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────────────┐
│                    Memory Layout                                   │
│                                                                    │
│  Stack:       1 KB (256 values × 4 bytes)                         │
│  Variables:   Dynamic (BTreeMap, grows as needed)                 │
│  Bytecode:    Dynamic (~8 bytes per instruction)                  │
│  Tokens:      Temporary (freed after parsing)                     │
│                                                                    │
│  Total:       ~2-10 KB typical (depends on program size)          │
└───────────────────────────────────────────────────────────────────┘

┌───────────────────────────────────────────────────────────────────┐
│                    OpCode Examples                                 │
│                                                                    │
│  Stack Operations:                                                │
│  ├─ Constant(n)        - Push constant to stack                  │
│  ├─ LoadVar(name)      - Push variable value to stack            │
│  ├─ StoreVar(name)     - Pop and store in variable               │
│  └─ Pop                - Discard top of stack                     │
│                                                                    │
│  Arithmetic:                                                      │
│  ├─ Add, Subtract      - Binary operations                       │
│  ├─ Multiply, Divide   - Binary operations                       │
│  ├─ Modulo             - Remainder operation                     │
│  └─ Negate             - Unary negation                          │
│                                                                    │
│  Comparison:                                                      │
│  ├─ Equal, NotEqual    - Equality checks                         │
│  ├─ Less, Greater      - Magnitude comparisons                   │
│  └─ LessEqual, GreaterEqual                                      │
│                                                                    │
│  Control Flow:                                                    │
│  ├─ Jump(addr)         - Unconditional jump                      │
│  └─ JumpIfFalse(addr)  - Conditional jump                        │
│                                                                    │
│  Built-ins:                                                       │
│  ├─ Print              - Print top of stack                      │
│  └─ Clear              - Clear VGA buffer                        │
└───────────────────────────────────────────────────────────────────┘
```

## Execution Example

```
Source:  let x = 5 + 3;
         print(x);

Tokens:  [Let, Identifier("x"), Equal, Number(5), Plus, Number(3), 
          Semicolon, Print, LeftParen, Identifier("x"), RightParen, 
          Semicolon, Eof]

Bytecode: [Constant(5), Constant(3), Add, StoreVar("x"), 
           LoadVar("x"), Print]

Stack Execution:
  1. Constant(5)       → Stack: [5]
  2. Constant(3)       → Stack: [5, 3]
  3. Add               → Stack: [8]
  4. StoreVar("x")     → Stack: [], Variables: {"x": 8}
  5. LoadVar("x")      → Stack: [8]
  6. Print             → Output: "8", Stack: []

Output: 8
```

## Integration Points

```
┌────────────────────────────────────────────────────┐
│  RustrialOS Components Used by Interpreter         │
├────────────────────────────────────────────────────┤
│                                                    │
│  1. println!() macro                              │
│     └─ Output values to VGA buffer                │
│                                                    │
│  2. WRITER.lock().clear_screen()                  │
│     └─ Clear VGA text buffer                      │
│                                                    │
│  3. alloc::collections::BTreeMap                  │
│     └─ Variable storage                           │
│                                                    │
│  4. alloc::vec::Vec                               │
│     └─ Token and bytecode storage                 │
│                                                    │
│  5. alloc::string::String                         │
│     └─ Variable names                             │
│                                                    │
└────────────────────────────────────────────────────┘
```
