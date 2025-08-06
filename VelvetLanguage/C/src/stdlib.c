#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <math.h>
#include "stdlib1.h"
#include "ast.h"

// Standard library function implementations
static void std_println(AstNodeList* args) {
    if (args && args->node) {
        // Simple implementation - just print the first argument
        if (args->node->type == AST_LITERAL) {
            if (args->node->literal.str_val[0] != '\0') {
                printf("%s\n", args->node->literal.str_val);
            } else if (args->node->literal.int_val != 0) {
                printf("%d\n", args->node->literal.int_val);
            } else if (args->node->literal.float_val != 0.0) {
                printf("%f\n", args->node->literal.float_val);
            } else if (args->node->literal.bool_val != 0) {
                printf("true\n");
            } else {
                printf("false\n");
            }
        } else if (args->node->type == AST_IDENTIFIER) {
            printf("%s\n", args->node->identifier.name);
        }
    } else {
        printf("\n");
    }
}

static void std_print(AstNodeList* args) {
    if (args && args->node) {
        if (args->node->type == AST_LITERAL) {
            if (args->node->literal.str_val[0] != '\0') {
                printf("%s", args->node->literal.str_val);
            } else if (args->node->literal.int_val != 0) {
                printf("%d", args->node->literal.int_val);
            } else if (args->node->literal.float_val != 0.0) {
                printf("%f", args->node->literal.float_val);
            } else if (args->node->literal.bool_val != 0) {
                printf("true");
            } else {
                printf("false");
            }
        } else if (args->node->type == AST_IDENTIFIER) {
            printf("%s", args->node->identifier.name);
        }
    }
}

static void std_input(AstNodeList* args) {
    char buffer[256];
    if (fgets(buffer, sizeof(buffer), stdin) != NULL) {
        // Remove newline
        buffer[strcspn(buffer, "\n")] = 0;
        printf("Input: %s\n", buffer);
    }
}

static void std_random(AstNodeList* args) {
    int max = 100; // Default max value
    if (args && args->node && args->node->type == AST_LITERAL) {
        max = args->node->literal.int_val;
    }
    int random_num = rand() % max;
    printf("Random number: %d\n", random_num);
}

static void std_sqrt(AstNodeList* args) {
    if (args && args->node && args->node->type == AST_LITERAL) {
        double num = args->node->literal.float_val;
        if (num == 0.0) num = args->node->literal.int_val;
        double result = sqrt(num);
        printf("Square root: %f\n", result);
    }
}

static void std_pow(AstNodeList* args) {
    if (args && args->next && args->node && args->next->node) {
        double base = 0.0, exponent = 0.0;
        
        if (args->node->type == AST_LITERAL) {
            base = args->node->literal.float_val;
            if (base == 0.0) base = args->node->literal.int_val;
        }
        
        if (args->next->node->type == AST_LITERAL) {
            exponent = args->next->node->literal.float_val;
            if (exponent == 0.0) exponent = args->next->node->literal.int_val;
        }
        
        double result = pow(base, exponent);
        printf("Power: %f\n", result);
    }
}

static void std_len(AstNodeList* args) {
    if (args && args->node && args->node->type == AST_LITERAL) {
        int length = strlen(args->node->literal.str_val);
        printf("Length: %d\n", length);
    }
}

static void std_substr(AstNodeList* args) {
    if (args && args->next && args->next->next && 
        args->node && args->next->node && args->next->next->node) {
        
        char* str = args->node->literal.str_val;
        int start = args->next->node->literal.int_val;
        int length = args->next->next->node->literal.int_val;
        
        if (start >= 0 && start < strlen(str) && length > 0) {
            char result[256];
            strncpy(result, str + start, length);
            result[length] = '\0';
            printf("Substring: %s\n", result);
        }
    }
}

static void std_parse_int(AstNodeList* args) {
    if (args && args->node && args->node->type == AST_LITERAL) {
        char* str = args->node->literal.str_val;
        int result = atoi(str);
        printf("Parsed integer: %d\n", result);
    }
}

static void std_parse_float(AstNodeList* args) {
    if (args && args->node && args->node->type == AST_LITERAL) {
        char* str = args->node->literal.str_val;
        double result = atof(str);
        printf("Parsed float: %f\n", result);
    }
}

static void std_to_string(AstNodeList* args) {
    if (args && args->node) {
        char result[256];
        if (args->node->type == AST_LITERAL) {
            if (args->node->literal.int_val != 0) {
                sprintf(result, "%d", args->node->literal.int_val);
            } else if (args->node->literal.float_val != 0.0) {
                sprintf(result, "%f", args->node->literal.float_val);
            } else if (args->node->literal.bool_val != 0) {
                strcpy(result, "true");
            } else {
                strcpy(result, "false");
            }
            printf("String: %s\n", result);
        }
    }
}

// Function registry
typedef struct {
    char name[64];
    void (*func)(AstNodeList*);
} StdFunction;

static StdFunction std_functions[] = {
    {"println", std_println},
    {"print", std_print},
    {"input", std_input},
    {"random", std_random},
    {"sqrt", std_sqrt},
    {"pow", std_pow},
    {"len", std_len},
    {"substr", std_substr},
    {"parse_int", std_parse_int},
    {"parse_float", std_parse_float},
    {"to_string", std_to_string},
    {"", NULL} // End marker
};

// Find and execute a standard library function
int call_std_function(const char* name, AstNodeList* args) {
    for (int i = 0; std_functions[i].func != NULL; i++) {
        if (strcmp(std_functions[i].name, name) == 0) {
            std_functions[i].func(args);
            return 1; // Function found and called
        }
    }
    return 0; // Function not found
}

// Register standard functions (placeholder for future use)
void register_std_functions() {
    // This function can be used to initialize any global state
    // needed for standard library functions
    printf("Standard library functions registered\n");
}

// Get list of available standard functions
const char** get_std_function_names() {
    static const char* names[64];
    int count = 0;
    
    for (int i = 0; std_functions[i].func != NULL && count < 63; i++) {
        names[count++] = std_functions[i].name;
    }
    names[count] = NULL;
    
    return names;
}

// Check if a function name is a standard library function
int is_std_function(const char* name) {
    for (int i = 0; std_functions[i].func != NULL; i++) {
        if (strcmp(std_functions[i].name, name) == 0) {
            return 1;
        }
    }
    return 0;
}