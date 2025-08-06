# Velvet Language

A custom programming language with two syntax variants: **Vel** (beginner-friendly) and **Vex** (symbolic).

## Quick Start

```bash
# Build the compiler
make

# Run a Vel program
./vexlc.exe tests/examples/test_vel_simple.vel

# Run a Vex program
./vexlc.exe tests/examples/test_vex_syntax.vex
```

## Documentation

### C Implementation (Vel & Vex)
- **[Syntax Reference](C/docs/syntax.md)** - Complete syntax documentation for both Vel and Vex
- **[Language Reference](C/docs/LANGUAGE_REFERENCE.md)** - Detailed language reference
- **[Architecture](C/docs/ARCHITECTURE.md)** - Compiler architecture details
- **[Examples](C/docs/EXAMPLES.md)** - Code examples and tutorials
- **[Main Documentation](C/docs/README.md)** - Comprehensive overview

### Rust Implementation (Vel only)
- Documentation coming soon...

## Features

 **Implemented**
- Variable declarations (`bind`, `bindm`)
- Type system (inference and explicit types)
- Type casting (`expression@type`)
- Basic operators (`+`, `-`, `*`, `/`, `<`, `>`, `==`, `!=`, `&&`, `||`)
- Function calls (`print()`)
- Both Vel and Vex syntax

 **In Progress**
- Control flow (`if`, `while`, `do`)
- Function declarations
- Arrays and data structures

 **Planned**
- Code generation to C
- Standard library expansion
- Modules and imports
- Error recovery

## Syntax Examples

### Vel (Beginner-friendly)
```vel
bind x as i32 = 5
bind y as i8 = 6
bind z as i32 = 5 + 6@i32
print(z)
```

### Vex (Symbolic)
```vex
bind x := 42
bind y: int = 13
print(x + y)
```

## Project Structure

```
C/
├── include/          # Header files
├── src/             # Source files
├── tests/           # Test files
│   └── examples/    # Example programs
├── docs/            # Documentation
└── Makefile         # Build configuration
```

## Contributing

1. Read the [Architecture](docs/ARCHITECTURE.md) documentation
2. Check existing examples in `tests/examples/`
3. Follow the development guidelines in the main documentation

## License

[Add your license information here] 