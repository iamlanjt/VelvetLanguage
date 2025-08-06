#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "typecheck.h"
#include "ast.h"
#include "token.h"

typedef enum {
    TYPE_UNKNOWN,
    TYPE_INT,
    TYPE_FLOAT,
    TYPE_STRING,
    TYPE_BOOL,
    TYPE_VOID
} TypeInfo;

static TypeInfo get_expression_type(AstNode *node) {
    if (!node) return TYPE_UNKNOWN;
    
    switch (node->type) {
        case AST_LITERAL:
            if (node->literal.int_val != 0 || node->literal.float_val != 0.0) {
                return TYPE_INT;
            }
            if (node->literal.str_val[0] != '\0') {
                return TYPE_STRING;
            }
            if (node->literal.bool_val != 0) {
                return TYPE_BOOL;
            }
            return TYPE_INT; // Default to int for numbers
            
        case AST_IDENTIFIER:
            // TODO: Look up variable type from symbol table
            return TYPE_UNKNOWN;
            
        case AST_BIN_OP:
            {
                TypeInfo left = get_expression_type(node->bin_op.left);
                TypeInfo right = get_expression_type(node->bin_op.right);
                
                // Simple type checking for arithmetic operations
                if (strcmp(node->bin_op.op, "+") == 0 || 
                    strcmp(node->bin_op.op, "-") == 0 ||
                    strcmp(node->bin_op.op, "*") == 0 ||
                    strcmp(node->bin_op.op, "/") == 0) {
                    if (left == TYPE_INT && right == TYPE_INT) {
                        return TYPE_INT;
                    }
                    if (left == TYPE_FLOAT || right == TYPE_FLOAT) {
                        return TYPE_FLOAT;
                    }
                }
                
                // Comparison operators return bool
                if (strcmp(node->bin_op.op, "<") == 0 || 
                    strcmp(node->bin_op.op, ">") == 0 ||
                    strcmp(node->bin_op.op, "==") == 0 ||
                    strcmp(node->bin_op.op, "!=") == 0) {
                    return TYPE_BOOL;
                }
            }
            break;
            
        case AST_UN_OP:
            {
                TypeInfo expr_type = get_expression_type(node->un_op.expr);
                if (strcmp(node->un_op.op, "!") == 0) {
                    return TYPE_BOOL;
                }
                return expr_type;
            }
            break;
            
        case AST_FUNC_CALL:
            // TODO: Look up function return type
            return TYPE_UNKNOWN;
            
        default:
            return TYPE_UNKNOWN;
    }
    
    return TYPE_UNKNOWN;
}

static int typecheck_statement(AstNode *node) {
    if (!node) return 1;
    
    switch (node->type) {
        case AST_VAR_DECL:
            {
                TypeInfo declared_type = TYPE_UNKNOWN;
                if (node->var_decl.type) {
                    // TODO: Parse type annotation
                }
                
                TypeInfo value_type = TYPE_UNKNOWN;
                if (node->var_decl.value) {
                    value_type = get_expression_type(node->var_decl.value);
                }
                
                // Simple type compatibility check
                if (declared_type != TYPE_UNKNOWN && value_type != TYPE_UNKNOWN && 
                    declared_type != value_type) {
                    printf("Type error: Cannot assign %d to variable of type %d\n", 
                           value_type, declared_type);
                    return 0;
                }
            }
            break;
            
        case AST_ASSIGN:
            {
                TypeInfo value_type = get_expression_type(node->bin_op.right);
                // TODO: Check if left side is assignable and types match
            }
            break;
            
        case AST_IF:
            {
                TypeInfo cond_type = get_expression_type(node->if_stmt.cond);
                if (cond_type != TYPE_BOOL && cond_type != TYPE_UNKNOWN) {
                    printf("Type error: Condition must be boolean\n");
                    return 0;
                }
                
                if (!typecheck_statement(node->if_stmt.then_block)) return 0;
                if (node->if_stmt.else_block && !typecheck_statement(node->if_stmt.else_block)) return 0;
            }
            break;
            
        case AST_WHILE:
            {
                TypeInfo cond_type = get_expression_type(node->while_stmt.cond);
                if (cond_type != TYPE_BOOL && cond_type != TYPE_UNKNOWN) {
                    printf("Type error: While condition must be boolean\n");
                    return 0;
                }
                
                if (!typecheck_statement(node->while_stmt.body)) return 0;
            }
            break;
            
        case AST_DO:
            {
                if (!typecheck_statement(node->do_stmt.body)) return 0;
            }
            break;
            
        case AST_BLOCK:
            {
                AstNodeList *stmt = node->block.stmts;
                while (stmt) {
                    if (!typecheck_statement(stmt->node)) return 0;
                    stmt = stmt->next;
                }
            }
            break;
            
        case AST_FUNC_DECL:
            {
                if (node->func_decl.body && !typecheck_statement(node->func_decl.body)) return 0;
            }
            break;
            
        default:
            // For expressions, just check if they're valid
            get_expression_type(node);
            break;
    }
    
    return 1;
}

int typecheck_program(AstNode* root) {
    if (!root) return 1;
    
    if (root->type != AST_PROGRAM) {
        printf("Type error: Root must be a program\n");
        return 0;
    }
    
    AstNodeList *stmt = root->program.stmts;
    while (stmt) {
        if (!typecheck_statement(stmt->node)) return 0;
        stmt = stmt->next;
    }
    
    return 1;
} 