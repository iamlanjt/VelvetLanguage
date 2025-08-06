# Velvet Language Examples

This document contains examples demonstrating the features of the Velvet Language.

## Basic Examples

### Simple Variable Declaration (Vel)
**File:** `test_vel_simple.vel`
```vel
bind x as int = 5@i32
print(x)
```
**Output:** `5`

### Simple Variable Declaration (Vex)
**File:** `test_vex_syntax.vex`
```vex
bind x := 42
bind y: int = 13
print(x)
print(y)
print(x + y)
```
**Output:** `421355`

## Type Casting Examples

### Basic Type Casting
**File:** `test_simple_typecast.vel`
```vel
bind x as i32 = 5@i32
print(x)
```
**Output:** `5`

### Binary Operation with Type Casting
**File:** `test_binary_typecast.vel`
```vel
bind z as i32 = 5 + 6@i32
print(z)
```
**Output:** `11`

### Step-by-Step Type Casting
**File:** `test_step_by_step.vel`
```vel
bind x as i32 = 5
print(x)
bind y as i8 = 6
print(y)
```
**Output:** `56`

### Complex Type Casting
**File:** `test_type_cast.vel`
```vel
bind x as i32 = 5
bind y as i8 = 6
bind z as i32 = 5 + 6@i32
print(z)
```
**Output:** `11`

## Comprehensive Examples

### Mixed Vel and Vex Syntax
**File:** `test_comprehensive_fixed.vel`
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
**Output:** `113030`

### Math Operations
**File:** `test_math.vex`
```vex
bind x := 10
bind y := 5
print(x + y)
print(x - y)
print(x * y)
print(x / y)
```
**Output:** `15502`

### Simple Vex Program
**File:** `test_simple.vex`
```vex
bind x := 42
print(x)
```
**Output:** `42`

### Comprehensive Vex Program
**File:** `test_comprehensive.vex`
```vex
bind x := 42
bind y: int = 13
print(x)
print(y)
print(x + y)
```
**Output:** `421355`

## Advanced Examples

### Function Calls
```vel
bind x as i32 = 5
bind y as i32 = 10
print(x + y)
print(x * y)
```

### Type Inference with Explicit Casting
```vel
bind a := 10        // Type inference
bind b: int = 20    // Explicit type
bind result as i32 = a + b@i32  // Cast result
print(result)
```

### Multiple Operations
```vel
bind x as i32 = 5
bind y as i32 = 3
bind z as i32 = x * y + 2@i32
print(z)
```

## Error Examples

### Invalid Type Casting
```vel
bind x as i32 = 5@invalid_type  // Error: invalid type
```

### Missing Assignment
```vel
bind x as i32  // Error: missing assignment
```

### Invalid Expression
```vel
bind x as i32 = 5 +  // Error: incomplete expression
```

## Best Practices

### Use Descriptive Variable Names
```vel
bind user_age as i32 = 25
bind user_name as string = "John"
print(user_age)
```

### Group Related Statements
```vel
// Initialize variables
bind x as i32 = 5
bind y as i32 = 10

// Perform calculations
bind sum as i32 = x + y
bind product as i32 = x * y

// Display results
print(sum)
print(product)
```

### Use Type Casting Sparingly
```vel
bind x as i32 = 5
bind y as i8 = 6
// Only cast when necessary
bind result as i32 = x + y@i32
```

## Testing Your Code

### Running Examples
```bash
# Build the compiler
make

# Run a Vel example
./vexlc.exe tests/examples/test_vel_simple.vel

# Run a Vex example
./vexlc.exe tests/examples/test_vex_syntax.vex
```

### Creating Your Own Examples
1. Create a `.vel` or `.vex` file
2. Write your code using the syntax rules
3. Run with `./vexlc.exe your_file.vel`

### Debugging
- Check syntax carefully
- Ensure all statements end properly
- Verify type casting syntax
- Use `print()` to debug values

## File Organization

All example files are located in `tests/examples/`:

### Vel Examples
- `test_vel_simple.vel` - Basic Vel syntax
- `test_simple_typecast.vel` - Simple type casting
- `test_binary_typecast.vel` - Binary operations with casting
- `test_step_by_step.vel` - Step-by-step execution
- `test_type_cast.vel` - Complex type casting
- `test_comprehensive_fixed.vel` - Mixed syntax example

### Vex Examples
- `test_vex_syntax.vex` - Basic Vex syntax
- `test_simple.vex` - Simple Vex program
- `test_math.vex` - Math operations
- `test_comprehensive.vex` - Comprehensive Vex example

### Legacy Examples
- `test_comprehensive.vel` - Original comprehensive example
- `test_vel.vel` - Original Vel example 