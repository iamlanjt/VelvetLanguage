# Velvet Language Reference

## Syntax Overview

The Velvet Language supports two syntax variants: **Vel** (beginner-friendly) and **Vex** (symbolic).

## Variables

### Vel Syntax
```vel
bind x as i32 = 5           // Explicit type declaration
bindm y as number = 9       // Mutable variable
bind z := 42                // Type inference (Vex-style in Vel)
```

### Vex Syntax
```vex
bind x := 42                // Type inference
bind y: int = 13           // Explicit type declaration
bindm z: float = 3.14      // Mutable variable
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

### While Loops
```vel
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

## Functions

### Function Declarations (Vel)
```vel
-> function_name(param1 as i32, param2 as string) => i32 {
    // function body
}
```

### Function Declarations (Vex)
```vex
fn function_name(param1: int, param2: string): int {
    // function body
}
```

### Function Calls
```vel
print(value)           // Built-in print function
write(value)          // Built-in write function
function_name(arg1, arg2)  // Custom function call
```

## Arrays

### Array Literals
```vel
bind arr as array = [1, 2, 3, 4, 5]
bind empty as array = []
```

### Array Access
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

### Do Blocks
```vel
do {
    bind x as i32 = 5
    print(x)
}
```

## Comments

### Single-line Comments
```vel
// This is a comment
bind x as i32 = 5  // Inline comment
```

### Multi-line Comments
```vel
/*
This is a multi-line comment
It can span multiple lines
*/
```

## Examples

### Complete Vel Program
```vel
bind x as i32 = 5
bind y as i8 = 6
bind z as i32 = 5 + 6@i32
print(z)

bind a := 10
bind b: int = 20
print(a + b)

bind result as i32 = a + b@i32
print(result)
```

### Complete Vex Program
```vex
bind x := 42
bind y: int = 13
print(x)
print(y)
print(x + y)
```

## Error Handling

### Common Errors
- **Parse Error**: Invalid syntax
- **Type Error**: Type mismatch
- **Runtime Error**: Division by zero, etc.

### Debugging
The compiler provides detailed error messages with line numbers and token information.

## Best Practices

### Naming Conventions
- Use descriptive variable names
- Use snake_case for variables
- Use PascalCase for functions

### Code Organization
- Group related statements together
- Use comments to explain complex logic
- Keep functions small and focused

### Type Safety
- Use explicit types when possible
- Use type casting sparingly
- Validate input data

## Performance Considerations

### Memory Management
- Variables are automatically managed
- Large arrays should be used carefully
- Avoid infinite loops

### Optimization
- Use appropriate data types
- Minimize function calls in loops
- Use efficient algorithms 