# Haxe Script Core

This is a Haxe toolchain that mirrors RustrialScript lexer/parser and manages script assets without touching kernel logic. It gives three things:  exact token/opcode parity with Rust, validation of .rscript files with line-based errors, and deterministic script_loader.rs regeneration and scaffold for new scripts. It is dev tooling only; kernel build unchanged unless RUSTRIAL_HAXE_VALIDATE enabled.

## Tools and commands

All commands run from repo root.

### Lexer test

Interprets LexerTest against example scripts.

```
haxe tools/lexer_test.hxml
```

### Parser test

Interprets ParserTest against example scripts.

```
haxe tools/parser_test.hxml
```

### Validator

Batch validate all example scripts:

```
haxe tools/validate.hxml
```

Single file validation:

```
haxe tools/validate.hxml -- src/rustrial_script/examples/fibonacci.rscript
```

### Pipeline

Validate + regenerate script_loader.rs (prints diff, writes only if changed):

```
haxe tools/pipeline.hxml
```

Include all scripts (even previously commented out):

```
haxe tools/pipeline.hxml -- --include-all
```

Exclude script by name:

```
haxe tools/pipeline.hxml -- --exclude triangle
```

### Scaffold new script

Create new script, validate, regenerate loader:

```
haxe tools/pipeline.hxml -- new-script demo
```

Creates:

```
src/rustrial_script/examples/demo.rscript
```

### Optional build hook

Enable Haxe validation during cargo build:

```
RUSTRIAL_HAXE_VALIDATE=1 cargo build
```

## Notes

- Haxe tools mirror current Rust features only (Int/Bool/Nil, no strings/functions/arrays).
- When Rust lexer/parser change, update Haxe Lexer/Parser to match.
- Pipeline keeps order from script_loader.rs and preserves commented-out entries unless --include-all is used.
