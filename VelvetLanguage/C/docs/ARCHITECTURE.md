# Velvet Language Compiler Architecture

## Overview

The Velvet Language compiler is built using a traditional compiler architecture with the following components:

1. **Lexer** - Tokenizes source code
2. **Parser** - Builds Abstract Syntax Tree (AST)
3. **Evaluator** - Interprets AST directly
4. **Type Checker** - Performs static type analysis
5. **Code Generator** - Generates C code (planned)

## Component Details

### Lexer (`src/lexer.c`, `include/lexer.h`)

The lexer converts source code into a stream of tokens.

**Key Functions:**
- `lexer_init()` - Initialize lexer with source code
- `lexer_next()` - Get next token
- `lexer_token_type_str()` - Convert token type to string

**Token Types:**
- Keywords: `bind`, `bindm`, `fn`, `if`, `while`, `do`, `else`
- Operators: `+`, `-`, `*`, `/`, `<`, `>`, `==`, `!=`, `&&`, `||`
- Type annotations: `as`, `:`, `:=`
- Type casting: `@`
- Literals: numbers, strings, booleans

### Parser (`src/parser.c`, `include/parser.h`)

The parser builds an Abstract Syntax Tree from tokens.

**Key Functions:**
- `parse_program()` - Main entry point
- `parse_statement()` - Parse individual statements
- `parse_expression()` - Parse expressions with operator precedence
- `parse_var_decl()` - Parse variable declarations
- `parse_func_decl()` - Parse function declarations

**AST Node Types:**
- `AST_PROGRAM` - Root program node
- `AST_VAR_DECL` - Variable declaration
- `AST_FUNC_DECL` - Function declaration
- `AST_FUNC_CALL` - Function call
- `AST_BIN_OP` - Binary operation
- `AST_UN_OP` - Unary operation
- `AST_TYPE_CAST` - Type casting
- `AST_LITERAL` - Literal values
- `AST_IDENTIFIER` - Variable references
- `AST_IF` - If statements
- `AST_WHILE` - While loops
- `AST_DO` - Do blocks
- `AST_BLOCK` - Code blocks

### AST (`include/ast.h`, `src/ast.c`)

The Abstract Syntax Tree represents the program structure.

**Key Structures:**
```c
typedef struct AstNode {
    AstNodeType type;
    union {
        struct { AstNodeList *stmts; } program;
        struct { char name[64]; AstNode *type; AstNode *value; int is_mut; } var_decl;
        struct { char name[64]; AstNodeList *args; } func_call;
        struct { AstNode *left; AstNode *right; char op[4]; } bin_op;
        struct { AstNode *expr; char target_type[16]; } type_cast;
        // ... other node types
    };
} AstNode;
```

**Memory Management:**
- `create_ast_node()` - Create new AST node
- `free_ast()` - Free AST memory
- `print_ast_node()` - Debug AST printing

### Evaluator (`src/eval.c`, `include/eval.h`)

The evaluator interprets the AST directly.

**Key Functions:**
- `eval_program()` - Evaluate entire program
- `eval_statement()` - Evaluate individual statements
- `eval_expression()` - Evaluate expressions

**Value System:**
```c
typedef union {
    int int_val;
    float float_val;
    char str_val[128];
    int bool_val;
} Value;
```

### Type Checker (`src/typecheck.c`, `include/typecheck.h`)

Performs static type analysis on the AST.

**Key Functions:**
- `typecheck_program()` - Type check entire program
- `typecheck_expression()` - Type check expressions
- `typecheck_statement()` - Type check statements

### Standard Library (`src/stdlib.c`, `include/stdlib1.h`)

Provides built-in functions.

**Available Functions:**
- `print()` - Print value to stdout
- `write()` - Write value to stdout
- Additional functions planned

## File Structure

```
C/
├── include/              # Header files
│   ├── ast.h            # AST definitions
│   ├── lexer.h          # Lexer interface
│   ├── parser.h         # Parser interface
│   ├── eval.h           # Evaluator interface
│   ├── typecheck.h      # Type checker interface
│   ├── compiler.h       # Code generator interface
│   ├── stdlib1.h        # Standard library
│   ├── token.h          # Token definitions
│   ├── utils.h          # Utility functions
│   └── src/             # Third-party libraries
│       ├── utarray.h    # Dynamic arrays
│       ├── uthash.h     # Hash tables
│       └── ...
├── src/                 # Source files
│   ├── main.c          # Entry point
│   ├── lexer.c         # Lexer implementation
│   ├── parser.c        # Parser implementation
│   ├── ast.c           # AST operations
│   ├── eval.c          # Evaluator implementation
│   ├── typecheck.c     # Type checker implementation
│   ├── compiler.c      # Code generator (planned)
│   ├── stdlib.c        # Standard library implementation
│   ├── token.c         # Token utilities
│   └── utils.c         # Utility functions
├── tests/              # Test files
│   ├── examples/       # Example programs
│   ├── test_lexer.c    # Lexer tests
│   ├── test_parser.c   # Parser tests
│   └── test_eval.c     # Evaluator tests
├── docs/               # Documentation
├── Makefile            # Build configuration
└── README.md           # Project overview
```

## Build Process

### Compilation Steps
1. **Preprocessing** - Header inclusion and macro expansion
2. **Compilation** - Source files compiled to object files
3. **Linking** - Object files linked into executable

### Makefile Targets
- `make` - Build the compiler
- `make clean` - Remove build artifacts
- `make test` - Run tests (planned)

## Error Handling

### Lexer Errors
- Invalid characters
- Unterminated strings
- Invalid number literals

### Parser Errors
- Unexpected tokens
- Missing tokens
- Invalid syntax

### Runtime Errors
- Division by zero
- Type mismatches
- Undefined variables

## Performance Considerations

### Memory Management
- AST nodes allocated dynamically
- Proper cleanup with `free_ast()`
- String handling with fixed-size buffers

### Optimization Opportunities
- Constant folding
- Dead code elimination
- Function inlining
- Loop optimization

## Future Enhancements

### Planned Features
- **Code Generation** - Generate C code
- **Optimization Passes** - Multiple optimization phases
- **Error Recovery** - Better error messages and recovery
- **Modules** - Import/export system
- **Standard Library** - Extended built-in functions

### Architecture Improvements
- **Symbol Table** - Better variable management
- **Scope Management** - Proper scoping rules
- **Type System** - More sophisticated type checking
- **Error Reporting** - Better error locations and messages 