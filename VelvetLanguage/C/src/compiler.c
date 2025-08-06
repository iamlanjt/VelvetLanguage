#include "compiler.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>

// Simple code generation to C
static void generate_expression_c(AstNode* node, FILE* output, int indent) {
    if (!node) return;
    
    for (int i = 0; i < indent; i++) {
        fprintf(output, "  ");
    }
    
    switch (node->type) {
        case AST_LITERAL:
            if (node->literal.int_val != 0) {
                fprintf(output, "%d", node->literal.int_val);
            } else if (node->literal.float_val != 0.0) {
                fprintf(output, "%f", node->literal.float_val);
            } else if (node->literal.str_val[0] != '\0') {
                fprintf(output, "\"%s\"", node->literal.str_val);
            } else if (node->literal.bool_val != 0) {
                fprintf(output, "1");
            } else {
                fprintf(output, "0");
            }
            break;
            
        case AST_IDENTIFIER:
            fprintf(output, "%s", node->identifier.name);
            break;
            
        case AST_BIN_OP:
            generate_expression_c(node->bin_op.left, output, 0);
            fprintf(output, " %s ", node->bin_op.op);
            generate_expression_c(node->bin_op.right, output, 0);
            break;
            
        case AST_UN_OP:
            fprintf(output, "%s", node->un_op.op);
            generate_expression_c(node->un_op.expr, output, 0);
            break;
            
        case AST_FUNC_CALL:
            fprintf(output, "%s(", node->func_call.name);
            if (node->func_call.args) {
                AstNodeList* arg = node->func_call.args;
                while (arg) {
                    generate_expression_c(arg->node, output, 0);
                    if (arg->next) fprintf(output, ", ");
                    arg = arg->next;
                }
            }
            fprintf(output, ")");
            break;
            
        default:
            fprintf(output, "/* unknown expression */");
            break;
    }
}

static void generate_statement_c(AstNode* node, FILE* output, int indent) {
    if (!node) return;
    
    for (int i = 0; i < indent; i++) {
        fprintf(output, "  ");
    }
    
    switch (node->type) {
        case AST_VAR_DECL:
            fprintf(output, "int %s", node->var_decl.name);
            if (node->var_decl.value) {
                fprintf(output, " = ");
                generate_expression_c(node->var_decl.value, output, 0);
            }
            fprintf(output, ";\n");
            break;
            
        case AST_ASSIGN:
            generate_expression_c(node->bin_op.left, output, 0);
            fprintf(output, " = ");
            generate_expression_c(node->bin_op.right, output, 0);
            fprintf(output, ";\n");
            break;
            
        case AST_BLOCK:
            fprintf(output, "{\n");
            {
                AstNodeList* stmt = node->block.stmts;
                while (stmt) {
                    generate_statement_c(stmt->node, output, indent + 1);
                    stmt = stmt->next;
                }
            }
            for (int i = 0; i < indent; i++) {
                fprintf(output, "  ");
            }
            fprintf(output, "}\n");
            break;
            
        case AST_IF:
            fprintf(output, "if (");
            generate_expression_c(node->if_stmt.cond, output, 0);
            fprintf(output, ") ");
            if (node->if_stmt.then_block->type == AST_BLOCK) {
                generate_statement_c(node->if_stmt.then_block, output, 0);
            } else {
                fprintf(output, "\n");
                generate_statement_c(node->if_stmt.then_block, output, indent + 1);
            }
            if (node->if_stmt.else_block) {
                for (int i = 0; i < indent; i++) {
                    fprintf(output, "  ");
                }
                fprintf(output, "else ");
                if (node->if_stmt.else_block->type == AST_BLOCK) {
                    generate_statement_c(node->if_stmt.else_block, output, 0);
                } else {
                    fprintf(output, "\n");
                    generate_statement_c(node->if_stmt.else_block, output, indent + 1);
                }
            }
            break;
            
        case AST_WHILE:
            fprintf(output, "while (");
            generate_expression_c(node->while_stmt.cond, output, 0);
            fprintf(output, ") ");
            if (node->while_stmt.body->type == AST_BLOCK) {
                generate_statement_c(node->while_stmt.body, output, 0);
            } else {
                fprintf(output, "\n");
                generate_statement_c(node->while_stmt.body, output, indent + 1);
            }
            break;
            
        case AST_DO:
            fprintf(output, "do ");
            if (node->do_stmt.body->type == AST_BLOCK) {
                generate_statement_c(node->do_stmt.body, output, 0);
            } else {
                fprintf(output, "\n");
                generate_statement_c(node->do_stmt.body, output, indent + 1);
            }
            fprintf(output, " while (0);\n"); // TODO: Add proper condition
            break;
            
        case AST_FUNC_DECL:
            fprintf(output, "void %s(", node->func_decl.name);
            if (node->func_decl.params) {
                AstNodeList* param = node->func_decl.params;
                while (param) {
                    fprintf(output, "int %s", param->node->identifier.name);
                    if (param->next) fprintf(output, ", ");
                    param = param->next;
                }
            }
            fprintf(output, ") ");
            if (node->func_decl.body) {
                generate_statement_c(node->func_decl.body, output, 0);
            } else {
                fprintf(output, ";\n");
            }
            break;
            
        case AST_FUNC_CALL:
            generate_expression_c(node, output, 0);
            fprintf(output, ";\n");
            break;
            
        default:
            generate_expression_c(node, output, 0);
            fprintf(output, ";\n");
            break;
    }
}

static void generate_program_c(AstNode* root, FILE* output) {
    fprintf(output, "#include <stdio.h>\n");
    fprintf(output, "#include <stdlib.h>\n\n");
    
    // Generate function declarations first
    if (root->type == AST_PROGRAM) {
        AstNodeList* stmt = root->program.stmts;
        while (stmt) {
            if (stmt->node->type == AST_FUNC_DECL) {
                fprintf(output, "void %s(", stmt->node->func_decl.name);
                if (stmt->node->func_decl.params) {
                    AstNodeList* param = stmt->node->func_decl.params;
                    while (param) {
                        fprintf(output, "int %s", param->node->identifier.name);
                        if (param->next) fprintf(output, ", ");
                        param = param->next;
                    }
                }
                fprintf(output, ");\n");
            }
            stmt = stmt->next;
        }
        fprintf(output, "\n");
        
        // Generate main function
        fprintf(output, "int main() {\n");
        stmt = root->program.stmts;
        while (stmt) {
            if (stmt->node->type != AST_FUNC_DECL) {
                generate_statement_c(stmt->node, output, 1);
            }
            stmt = stmt->next;
        }
        fprintf(output, "  return 0;\n");
        fprintf(output, "}\n\n");
        
        // Generate function definitions
        stmt = root->program.stmts;
        while (stmt) {
            if (stmt->node->type == AST_FUNC_DECL) {
                generate_statement_c(stmt->node, output, 0);
                fprintf(output, "\n");
            }
            stmt = stmt->next;
        }
    }
}

// Main compilation function
void compile_program(AstNode* root) {
    if (!root) {
        printf("Error: No AST to compile\n");
        return;
    }
    
    FILE* output = fopen("output.c", "w");
    if (!output) {
        printf("Error: Cannot create output file\n");
        return;
    }
    
    generate_program_c(root, output);
    fclose(output);
    
    printf("Compilation successful. Generated output.c\n");
}

// Compile to a specific output file
void compile_program_to_file(AstNode* root, const char* filename) {
    if (!root) {
        printf("Error: No AST to compile\n");
        return;
    }
    
    FILE* output = fopen(filename, "w");
    if (!output) {
        printf("Error: Cannot create output file '%s'\n", filename);
        return;
    }
    
    generate_program_c(root, output);
    fclose(output);
    
    printf("Compilation successful. Generated %s\n", filename);
}
