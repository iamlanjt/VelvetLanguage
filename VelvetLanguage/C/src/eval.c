#include "eval.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <math.h>

// Simple value representation for evaluation
typedef struct {
    enum {
        VAL_INT,
        VAL_FLOAT,
        VAL_STRING,
        VAL_BOOL,
        VAL_VOID
    } type;
    union {
        int int_val;
        double float_val;
        char str_val[128];
        int bool_val;
    };
} Value;

// Simple environment for variable storage
typedef struct Env {
    char name[64];
    Value value;
    struct Env* next;
} Env;

static Env* global_env = NULL;

// Create a new environment entry
static Env* create_env_entry(const char* name, Value value) {
    Env* env = malloc(sizeof(Env));
    if (!env) {
        fprintf(stderr, "Memory allocation failed for environment\n");
        exit(1);
    }
    strncpy(env->name, name, 63);
    env->name[63] = '\0';
    env->value = value;
    env->next = NULL;
    return env;
}

// Set a variable in the environment
static void set_variable(const char* name, Value value) {
    // Check if variable already exists
    Env* current = global_env;
    while (current) {
        if (strcmp(current->name, name) == 0) {
            current->value = value;
            return;
        }
        current = current->next;
    }
    
    // Create new variable
    Env* new_env = create_env_entry(name, value);
    new_env->next = global_env;
    global_env = new_env;
}

// Get a variable from the environment
static Value* get_variable(const char* name) {
    Env* current = global_env;
    while (current) {
        if (strcmp(current->name, name) == 0) {
            return &current->value;
        }
        current = current->next;
    }
    return NULL;
}

// Create a value
static Value create_int_value(int val) {
    Value v = {VAL_INT, {.int_val = val}};
    return v;
}

static Value create_float_value(double val) {
    Value v = {VAL_FLOAT, {.float_val = val}};
    return v;
}

static Value create_string_value(const char* val) {
    Value v = {VAL_STRING, {.str_val = {0}}};
    strncpy(v.str_val, val, 127);
    v.str_val[127] = '\0';
    return v;
}

static Value create_bool_value(int val) {
    Value v = {VAL_BOOL, {.bool_val = val}};
    return v;
}

static Value create_void_value() {
    Value v = {VAL_VOID, {0}};
    return v;
}

// Print a value
static void print_value(Value* val) {
    switch (val->type) {
        case VAL_INT:
            printf("%d", val->int_val);
            break;
        case VAL_FLOAT:
            printf("%f", val->float_val);
            break;
        case VAL_STRING:
            printf("%s", val->str_val);
            break;
        case VAL_BOOL:
            printf(val->bool_val ? "true" : "false");
            break;
        case VAL_VOID:
            printf("void");
            break;
    }
}

// Evaluate an expression
static Value eval_expression(AstNode* node) {
    if (!node) return create_void_value();
    
    switch (node->type) {
        case AST_LITERAL:
            if (node->literal.int_val != 0) {
                return create_int_value(node->literal.int_val);
            }
            if (node->literal.float_val != 0.0) {
                return create_float_value(node->literal.float_val);
            }
            if (node->literal.str_val[0] != '\0') {
                return create_string_value(node->literal.str_val);
            }
            if (node->literal.bool_val != 0) {
                return create_bool_value(node->literal.bool_val);
            }
            return create_int_value(0);
            
        case AST_IDENTIFIER: {
            Value* var = get_variable(node->identifier.name);
            if (var) {
                return *var;
            } else {
                printf("Error: Undefined variable '%s'\n", node->identifier.name);
                return create_void_value();
            }
        }
            
        case AST_BIN_OP: {
            Value left = eval_expression(node->bin_op.left);
            Value right = eval_expression(node->bin_op.right);
            Value result;
            
            if (strcmp(node->bin_op.op, "+") == 0) {
                if (left.type == VAL_INT && right.type == VAL_INT) {
                    result = create_int_value(left.int_val + right.int_val);
                } else if (left.type == VAL_FLOAT || right.type == VAL_FLOAT) {
                    double l = (left.type == VAL_INT) ? left.int_val : left.float_val;
                    double r = (right.type == VAL_INT) ? right.int_val : right.float_val;
                    result = create_float_value(l + r);
                } else if (left.type == VAL_STRING || right.type == VAL_STRING) {
                    // String concatenation
                    char temp[256];
                    if (left.type == VAL_STRING) {
                        strcpy(temp, left.str_val);
                    } else {
                        sprintf(temp, "%d", left.int_val);
                    }
                    if (right.type == VAL_STRING) {
                        strcat(temp, right.str_val);
                    } else {
                        char temp2[64];
                        sprintf(temp2, "%d", right.int_val);
                        strcat(temp, temp2);
                    }
                    result = create_string_value(temp);
                } else {
                    result = create_int_value(left.int_val + right.int_val);
                }
            } else if (strcmp(node->bin_op.op, "-") == 0) {
                if (left.type == VAL_FLOAT || right.type == VAL_FLOAT) {
                    double l = (left.type == VAL_INT) ? left.int_val : left.float_val;
                    double r = (right.type == VAL_INT) ? right.int_val : right.float_val;
                    result = create_float_value(l - r);
                } else {
                    result = create_int_value(left.int_val - right.int_val);
                }
            } else if (strcmp(node->bin_op.op, "*") == 0) {
                if (left.type == VAL_FLOAT || right.type == VAL_FLOAT) {
                    double l = (left.type == VAL_INT) ? left.int_val : left.float_val;
                    double r = (right.type == VAL_INT) ? right.int_val : right.float_val;
                    result = create_float_value(l * r);
                } else {
                    result = create_int_value(left.int_val * right.int_val);
                }
            } else if (strcmp(node->bin_op.op, "/") == 0) {
                if (right.int_val == 0) {
                    printf("Error: Division by zero\n");
                    return create_void_value();
                }
                if (left.type == VAL_FLOAT || right.type == VAL_FLOAT) {
                    double l = (left.type == VAL_INT) ? left.int_val : left.float_val;
                    double r = (right.type == VAL_INT) ? right.int_val : right.float_val;
                    result = create_float_value(l / r);
                } else {
                    result = create_int_value(left.int_val / right.int_val);
                }
            } else if (strcmp(node->bin_op.op, "<") == 0) {
                int l = (left.type == VAL_INT) ? left.int_val : (int)left.float_val;
                int r = (right.type == VAL_INT) ? right.int_val : (int)right.float_val;
                result = create_bool_value(l < r);
            } else if (strcmp(node->bin_op.op, ">") == 0) {
                int l = (left.type == VAL_INT) ? left.int_val : (int)left.float_val;
                int r = (right.type == VAL_INT) ? right.int_val : (int)right.float_val;
                result = create_bool_value(l > r);
            } else if (strcmp(node->bin_op.op, "==") == 0) {
                if (left.type == VAL_STRING && right.type == VAL_STRING) {
                    result = create_bool_value(strcmp(left.str_val, right.str_val) == 0);
                } else {
                    int l = (left.type == VAL_INT) ? left.int_val : (int)left.float_val;
                    int r = (right.type == VAL_INT) ? right.int_val : (int)right.float_val;
                    result = create_bool_value(l == r);
                }
            } else if (strcmp(node->bin_op.op, "!=") == 0) {
                if (left.type == VAL_STRING && right.type == VAL_STRING) {
                    result = create_bool_value(strcmp(left.str_val, right.str_val) != 0);
                } else {
                    int l = (left.type == VAL_INT) ? left.int_val : (int)left.float_val;
                    int r = (right.type == VAL_INT) ? right.int_val : (int)right.float_val;
                    result = create_bool_value(l != r);
                }
            } else {
                printf("Error: Unknown binary operator '%s'\n", node->bin_op.op);
                return create_void_value();
            }
            return result;
        }
            
        case AST_UN_OP: {
            Value expr = eval_expression(node->un_op.expr);
            if (strcmp(node->un_op.op, "!") == 0) {
                return create_bool_value(!expr.bool_val);
            } else if (strcmp(node->un_op.op, "-") == 0) {
                if (expr.type == VAL_INT) {
                    return create_int_value(-expr.int_val);
                } else if (expr.type == VAL_FLOAT) {
                    return create_float_value(-expr.float_val);
                }
            }
            printf("Error: Unknown unary operator '%s'\n", node->un_op.op);
            return create_void_value();
        }
            
        case AST_TYPE_CAST: {
            Value expr_val = eval_expression(node->type_cast.expr);
            // For now, just return the expression value
            // In a full implementation, you'd perform the actual type conversion
            return expr_val;
        }
            
        case AST_ASSIGN: {
            Value value = eval_expression(node->var_decl.value);
            set_variable(node->var_decl.name, value);
            return value;
        }
            
        case AST_FUNC_CALL: {
            if (strcmp(node->func_call.name, "println") == 0) {
                if (node->func_call.args && node->func_call.args->node) {
                    Value arg = eval_expression(node->func_call.args->node);
                    print_value(&arg);
                    printf("\n");
                }
                return create_void_value();
            } else if (strcmp(node->func_call.name, "print") == 0) {
                if (node->func_call.args && node->func_call.args->node) {
                    Value arg = eval_expression(node->func_call.args->node);
                    print_value(&arg);
                }
                return create_void_value();
            } else {
                printf("Error: Unknown function '%s'\n", node->func_call.name);
                return create_void_value();
            }
        }
            
        default:
            printf("Error: Cannot evaluate expression type %d\n", node->type);
            return create_void_value();
    }
}

// Evaluate a statement
static Value eval_statement(AstNode* node) {
    if (!node) return create_void_value();
    
    switch (node->type) {
        case AST_VAR_DECL: {
            Value value = create_void_value();
            if (node->var_decl.value) {
                value = eval_expression(node->var_decl.value);
            }
            set_variable(node->var_decl.name, value);
            return create_void_value();
        }
            
        case AST_BLOCK: {
            AstNodeList* stmt = node->block.stmts;
            Value result = create_void_value();
            while (stmt) {
                result = eval_statement(stmt->node);
                stmt = stmt->next;
            }
            return result;
        }
            
        case AST_IF: {
            Value cond = eval_expression(node->if_stmt.cond);
            if (cond.bool_val) {
                return eval_statement(node->if_stmt.then_block);
            } else if (node->if_stmt.else_block) {
                return eval_statement(node->if_stmt.else_block);
            }
            return create_void_value();
        }
            
        case AST_WHILE: {
            Value result = create_void_value();
            while (1) {
                Value cond = eval_expression(node->while_stmt.cond);
                if (!cond.bool_val) break;
                result = eval_statement(node->while_stmt.body);
            }
            return result;
        }
            
        case AST_DO: {
            Value result = create_void_value();
            do {
                result = eval_statement(node->do_stmt.body);
            } while (0); // TODO: Add condition for do-while
            return result;
        }
            
        case AST_FUNC_DECL:
            // Function declarations are stored in environment for now
            return create_void_value();
            
        default:
            return eval_expression(node);
    }
}

// Main evaluation function
void eval_program(AstNode* root) {
    if (!root) return;
    
    if (root->type != AST_PROGRAM) {
        printf("Error: Root must be a program\n");
        return;
    }
    
    AstNodeList* stmt = root->program.stmts;
    while (stmt) {
        eval_statement(stmt->node);
        stmt = stmt->next;
    }
}

// Clean up environment
void cleanup_eval() {
    while (global_env) {
        Env* next = global_env->next;
        free(global_env);
        global_env = next;
    }
}