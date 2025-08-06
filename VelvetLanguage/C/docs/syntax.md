# Velvet Language Syntax Reference

This document provides a comprehensive reference for the syntax of both Vel and Vex languages in the Velvet Language project.

## Language Variants

- **Vel**: Beginner-friendly, spelled-out syntax
- **Vex**: Symbolic, C-like syntax

## Comments

### Single-line Comments
```vel
// This is a comment
bind x as i32 = 5  // Inline comment
```

```vex
// This is a comment
bind x := 42  // Inline comment
```

### Multi-line Comments
```vel
/*
This is a multi-line comment
It can span multiple lines
*/
```

```vex
/*
This is a multi-line comment
It can span multiple lines
*/
```

## Variables

### Variable Declarations

#### Vel Syntax
```vel
bind x as i32 = 5           // Explicit type declaration
bindm y as number = 9       // Mutable variable
bind z := 42                // Type inference (Vex-style in Vel)
```

#### Vex Syntax
```vex
bind x := 42                // Type inference
bind y: int = 13           // Explicit type declaration
bindm z: float = 3.14      // Mutable variable
```

### Assignment
```vel
x = 10                     // Assignment to existing variable
```

```vex
x = 10                     // Assignment to existing variable
```

## Types

### Supported Types
- `int` / `i32` - 32-bit integer
- `i8` - 8-bit integer
- `float` / `number` - Floating point
- `string` / `str` - String
- `bool` - Boolean
- `any` - Dynamic type

## Type Casting

### Vel Syntax
```vel
bind x as i32 = 5@i32       // Cast literal to type
bind y as i32 = a + b@i32   // Cast expression to type
```

### Vex Syntax
```vex
bind x: int = 5@int         // Cast literal to type
bind y: int = a + b@int     // Cast expression to type
```

## Operators

### Arithmetic Operators
```vel
a + b    // Addition
a - b    // Subtraction
a * b    // Multiplication
a / b    // Division
```

### Comparison Operators
```vel
a < b    // Less than
a > b    // Greater than
a == b   // Equal to
a != b   // Not equal to
```

### Logical Operators
```vel
a && b   // Logical AND
a || b   // Logical OR
!a       // Logical NOT
```

### Nullish Coalescing
```vel
a ! b    // Returns a if a is not null, otherwise b
```

## Control Flow

### If Statements
```vel
if condition {
    // code
} else {
    // code
}
```

```vex
if condition {
    // code
} else {
    // code
}
```

### While Loops
```vel
while condition {
    // code
}
```

```vex
while condition {
    // code
}
```

### Do Blocks
```vel
do {
    // code
}
```

```vex
do {
    // code
}
```

## Functions

### Function Calls
```vel
print(value)           // Built-in print function
write(value)          // Built-in write function (Vex only)
function_name(arg1, arg2)  // Custom function call
```

```vex
print(value)           // Built-in print function
write(value)          // Built-in write function
function_name(arg1, arg2)  // Custom function call
```

### Function Declarations (Not Yet Implemented)

#### Vel Syntax (Planned)
```vel
-> function_name(param1 as i32, param2 as string) => i32 {
    // function body
}
```

#### Vex Syntax (Planned)
```vex
fn function_name(param1: int, param2: string): int {
    // function body
}
```

## Arrays (Not Yet Implemented)

### Array Literals (Planned)
```vel
bind arr as array = [1, 2, 3, 4, 5]
bind empty as array = []
```

### Array Access (Planned)
```vel
bind value as int = arr[0]    // Access element
arr[1] = 42                   // Assign element
```

## Blocks and Scopes

### Code Blocks
```vel
{
    bind x as i32 = 5
    print(x)
}
```

```vex
{
    bind x := 42
    print(x)
}
```

## Examples

### Complete Vel Program
```vel
// Test all implemented features
bind x as i32 = 5
bind y as i8 = 6
bind z as i32 = 5 + 6@i32
print(z)

// Test Vex-style syntax in Vel
bind a := 10
bind b: int = 20
print(a + b)

// Test type casting
bind result as i32 = a + b@i32
print(result)

// Test basic arithmetic
bind sum as i32 = x + y
bind diff as i32 = x - y
bind prod as i32 = x * y
print(sum)
print(diff)
print(prod)
```

### Complete Vex Program
```vex
bind x := 42
bind y: int = 13
print(x)
print(y)
print(x + y)
```

## Implementation Status

### ✅ Implemented Features

#### Lexer & Parser
- [x] Comments (`//`, `/* */`)
- [x] Variable declarations (`bind`, `bindm`)
- [x] Type annotations (`as type`, `: type`)
- [x] Type inference (`:=`)
- [x] Assignment expressions (`=`)
- [x] String literals (`"..."`)
- [x] Number literals
- [x] Type casting (`expression@type`)
- [x] Basic arithmetic operators (`+`, `-`, `*`, `/`)
- [x] Comparison operators (`<`, `>`, `==`, `!=`)
- [x] Logical operators (`&&`, `||`, `!`)
- [x] Function calls (`print()`, `write()`)
- [x] Control flow parsing (`if`, `while`, `do`)

#### Evaluator
- [x] Variable declarations and assignment
- [x] String literals
- [x] Number literals
- [x] Basic arithmetic operations
- [x] Comparison operations
- [x] Function calls (`print`)
- [x] Assignment expressions
- [x] Type casting (basic support)

### ❌ Missing Features

#### Control Flow (Parsed but not evaluated correctly)
- [ ] If statements evaluation
- [ ] While loops evaluation
- [ ] Do blocks evaluation

#### Function System
- [ ] Function declarations (Vel: `-> func() => type {}`)
- [ ] Function declarations (Vex: `fn func(): type {}`)
- [ ] Function parameters
- [ ] Return statements
- [ ] Function scope

#### Data Structures
- [ ] Array literals (`[1, 2, 3]`)
- [ ] Array access (`arr[0]`)
- [ ] Array assignment (`arr[0] = 42`)

#### Advanced Features
- [ ] Nullish coalescing (`a ! b`)
- [ ] Logical operators evaluation (`&&`, `||`, `!`)
- [ ] Standard library functions (`write()` for Vex)
- [ ] Error handling and recovery
- [ ] Type checking and validation

#### Code Generation
- [ ] C code generation
- [ ] LLVM IR generation (Rust version)
- [ ] Optimization passes

#### Language-Specific Features
- [ ] Vel-specific syntax (`->`, `=>`)
- [ ] Vex-specific syntax (`fn`, `:`)
- [ ] Import/export system (planned for Vex)

## Known Issues

1. **Control Flow Evaluation**: If statements and while loops are parsed correctly but not evaluated properly
2. **Function Declarations**: Not implemented yet
3. **Arrays**: Not implemented yet
4. **Logical Operators**: Parsed but not fully evaluated
5. **Type System**: Basic support only, no type checking

## Future Enhancements

1. **Complete Control Flow**: Fix if/while evaluation
2. **Function System**: Implement function declarations and calls
3. **Array Support**: Add array literals and access
4. **Type System**: Implement proper type checking
5. **Standard Library**: Expand built-in functions
6. **Error Handling**: Better error messages and recovery
7. **Code Generation**: Generate executable code
8. **Optimization**: Performance improvements

## Testing

Test files are located in `tests/examples/`:
- `test_features.vel` - Tests all implemented features
- `test_control_flow.vel` - Tests control flow (partially working)
- `test_simple_string.vel` - Tests string literals
- `test_debug_condition.vel` - Tests condition evaluation
- `test_vex_syntax.vex` - Tests Vex syntax
- `test_vel_simple.vel` - Tests basic Vel syntax

## Notes

- The language is currently in interpreter mode only
- Both Vel and Vex share the same underlying implementation
- Syntax differences are handled in the lexer and parser
- The evaluator treats both languages identically
- Type casting is supported but type checking is basic
- Error handling is minimal and needs improvement 