#include "ast.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

// Create a new AST node
AstNode* create_ast_node(AstNodeType type) {
    AstNode* node = malloc(sizeof(AstNode));
    if (!node) {
        fprintf(stderr, "Memory allocation failed for AST node\n");
        exit(1);
    }
    memset(node, 0, sizeof(AstNode));
    node->type = type;
    return node;
}

// Create a program node
AstNode* create_program_node(AstNodeList* statements) {
    AstNode* node = create_ast_node(AST_PROGRAM);
    node->program.stmts = statements;
    return node;
}

// Create a block node
AstNode* create_block_node(AstNodeList* statements) {
    AstNode* node = create_ast_node(AST_BLOCK);
    node->block.stmts = statements;
    return node;
}

// Create a variable declaration node
AstNode* create_var_decl_node(const char* name, int is_mut, AstNode* type, AstNode* value) {
    AstNode* node = create_ast_node(AST_VAR_DECL);
    strncpy(node->var_decl.name, name, 63);
    node->var_decl.name[63] = '\0';
    node->var_decl.is_mut = is_mut;
    node->var_decl.type = type;
    node->var_decl.value = value;
    return node;
}

// Create a function declaration node
AstNode* create_func_decl_node(const char* name, AstNodeList* params, AstNode* body) {
    AstNode* node = create_ast_node(AST_FUNC_DECL);
    strncpy(node->func_decl.name, name, 63);
    node->func_decl.name[63] = '\0';
    node->func_decl.params = params;
    node->func_decl.body = body;
    return node;
}

// Create a function call node
AstNode* create_func_call_node(const char* name, AstNodeList* args) {
    AstNode* node = create_ast_node(AST_FUNC_CALL);
    strncpy(node->func_call.name, name, 63);
    node->func_call.name[63] = '\0';
    node->func_call.args = args;
    return node;
}

// Create an if statement node
AstNode* create_if_node(AstNode* cond, AstNode* then_block, AstNode* else_block) {
    AstNode* node = create_ast_node(AST_IF);
    node->if_stmt.cond = cond;
    node->if_stmt.then_block = then_block;
    node->if_stmt.else_block = else_block;
    return node;
}

// Create a while loop node
AstNode* create_while_node(AstNode* cond, AstNode* body) {
    AstNode* node = create_ast_node(AST_WHILE);
    node->while_stmt.cond = cond;
    node->while_stmt.body = body;
    return node;
}

// Create a do-while loop node
AstNode* create_do_node(AstNode* body) {
    AstNode* node = create_ast_node(AST_DO);
    node->do_stmt.body = body;
    return node;
}

// Create a literal node
AstNode* create_literal_node(int int_val, double float_val, const char* str_val, int bool_val) {
    AstNode* node = create_ast_node(AST_LITERAL);
    node->literal.int_val = int_val;
    node->literal.float_val = float_val;
    if (str_val) {
        strncpy(node->literal.str_val, str_val, 127);
        node->literal.str_val[127] = '\0';
    }
    node->literal.bool_val = bool_val;
    return node;
}

// Create an identifier node
AstNode* create_identifier_node(const char* name) {
    AstNode* node = create_ast_node(AST_IDENTIFIER);
    strncpy(node->identifier.name, name, 63);
    node->identifier.name[63] = '\0';
    return node;
}

// Create a binary operation node
AstNode* create_bin_op_node(AstNode* left, AstNode* right, const char* op) {
    AstNode* node = create_ast_node(AST_BIN_OP);
    node->bin_op.left = left;
    node->bin_op.right = right;
    strncpy(node->bin_op.op, op, 3);
    node->bin_op.op[3] = '\0';
    return node;
}

// Create a unary operation node
AstNode* create_un_op_node(AstNode* expr, const char* op) {
    AstNode* node = create_ast_node(AST_UN_OP);
    node->un_op.expr = expr;
    strncpy(node->un_op.op, op, 3);
    node->un_op.op[3] = '\0';
    return node;
}

// Create an AST node list
AstNodeList* create_ast_node_list(AstNode* node) {
    AstNodeList* list = malloc(sizeof(AstNodeList));
    if (!list) {
        fprintf(stderr, "Memory allocation failed for AST node list\n");
        exit(1);
    }
    list->node = node;
    list->next = NULL;
    return list;
}

// Add a node to the end of an AST node list
AstNodeList* append_to_ast_list(AstNodeList* list, AstNode* node) {
    AstNodeList* new_item = create_ast_node_list(node);
    
    if (!list) {
        return new_item;
    }
    
    AstNodeList* current = list;
    while (current->next) {
        current = current->next;
    }
    current->next = new_item;
    return list;
}

// Free an AST node list
void free_ast_list(AstNodeList* list) {
    while (list) {
        AstNodeList* next = list->next;
        free_ast(list->node);
        free(list);
        list = next;
    }
}

// Free an AST node and all its children
void free_ast(AstNode* node) {
    if (!node) return;
    
    switch (node->type) {
        case AST_PROGRAM:
            free_ast_list(node->program.stmts);
            break;
            
        case AST_BLOCK:
            free_ast_list(node->block.stmts);
            break;
            
        case AST_VAR_DECL:
            free_ast(node->var_decl.type);
            free_ast(node->var_decl.value);
            break;
            
        case AST_FUNC_DECL:
            free_ast_list(node->func_decl.params);
            free_ast(node->func_decl.body);
            break;
            
        case AST_FUNC_CALL:
            free_ast_list(node->func_call.args);
            break;
            
        case AST_IF:
            free_ast(node->if_stmt.cond);
            free_ast(node->if_stmt.then_block);
            free_ast(node->if_stmt.else_block);
            break;
            
        case AST_WHILE:
            free_ast(node->while_stmt.cond);
            free_ast(node->while_stmt.body);
            break;
            
        case AST_DO:
            free_ast(node->do_stmt.body);
            break;
            
        case AST_BIN_OP:
            free_ast(node->bin_op.left);
            free_ast(node->bin_op.right);
            break;
            
        case AST_UN_OP:
            free_ast(node->un_op.expr);
            break;
            
        case AST_ASSIGN:
            free_ast(node->bin_op.left);
            free_ast(node->bin_op.right);
            break;
            
        case AST_TYPE_CAST:
            free_ast(node->type_cast.expr);
            break;
            
        case AST_LITERAL:
        case AST_IDENTIFIER:
            // No children to free
            break;
    }
    
    free(node);
}

// Print AST node for debugging
void print_ast_node(AstNode* node, int indent) {
    if (!node) return;
    
    for (int i = 0; i < indent; i++) {
        printf("  ");
    }
    
    switch (node->type) {
        case AST_PROGRAM:
            printf("Program\n");
            {
                AstNodeList* stmt = node->program.stmts;
                while (stmt) {
                    print_ast_node(stmt->node, indent + 1);
                    stmt = stmt->next;
                }
            }
            break;
            
        case AST_BLOCK:
            printf("Block\n");
            {
                AstNodeList* stmt = node->block.stmts;
                while (stmt) {
                    print_ast_node(stmt->node, indent + 1);
                    stmt = stmt->next;
                }
            }
            break;
            
        case AST_VAR_DECL:
            printf("VarDecl: %s (mut: %d)\n", node->var_decl.name, node->var_decl.is_mut);
            if (node->var_decl.type) print_ast_node(node->var_decl.type, indent + 1);
            if (node->var_decl.value) print_ast_node(node->var_decl.value, indent + 1);
            break;
            
        case AST_FUNC_DECL:
            printf("FuncDecl: %s\n", node->func_decl.name);
            if (node->func_decl.params) {
                AstNodeList* param = node->func_decl.params;
                while (param) {
                    print_ast_node(param->node, indent + 1);
                    param = param->next;
                }
            }
            if (node->func_decl.body) print_ast_node(node->func_decl.body, indent + 1);
            break;
            
        case AST_FUNC_CALL:
            printf("FuncCall: %s\n", node->func_call.name);
            if (node->func_call.args) {
                AstNodeList* arg = node->func_call.args;
                while (arg) {
                    print_ast_node(arg->node, indent + 1);
                    arg = arg->next;
                }
            }
            break;
            
        case AST_IF:
            printf("If\n");
            print_ast_node(node->if_stmt.cond, indent + 1);
            print_ast_node(node->if_stmt.then_block, indent + 1);
            if (node->if_stmt.else_block) print_ast_node(node->if_stmt.else_block, indent + 1);
            break;
            
        case AST_WHILE:
            printf("While\n");
            print_ast_node(node->while_stmt.cond, indent + 1);
            print_ast_node(node->while_stmt.body, indent + 1);
            break;
            
        case AST_DO:
            printf("Do\n");
            print_ast_node(node->do_stmt.body, indent + 1);
            break;
            
        case AST_LITERAL:
            printf("Literal: int=%d, float=%f, str='%s', bool=%d\n", 
                   node->literal.int_val, node->literal.float_val, 
                   node->literal.str_val, node->literal.bool_val);
            break;
            
        case AST_IDENTIFIER:
            printf("Identifier: %s\n", node->identifier.name);
            break;
            
        case AST_BIN_OP:
            printf("BinOp: %s\n", node->bin_op.op);
            print_ast_node(node->bin_op.left, indent + 1);
            print_ast_node(node->bin_op.right, indent + 1);
            break;
            
        case AST_UN_OP:
            printf("UnOp: %s\n", node->un_op.op);
            print_ast_node(node->un_op.expr, indent + 1);
            break;
            
        case AST_ASSIGN:
            printf("Assign\n");
            print_ast_node(node->bin_op.left, indent + 1);
            print_ast_node(node->bin_op.right, indent + 1);
            break;
            
        case AST_TYPE_CAST:
            printf("TypeCast: %s\n", node->type_cast.target_type);
            print_ast_node(node->type_cast.expr, indent + 1);
            break;
    }
}

// Note: parse_program is implemented in parser.c