#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "lexer.h"
#include "token.h"
#include "ast.h"

// Current token (ont touch iys global for a reason)
static Token cur_token;

// Advance to the next token
static void advance()
{
    cur_token = lexer_next();
}

// Match and use a token of expected shi
static int match(VexTokenType type)
{
    if (cur_token.type == type)
    {
        advance();
        return 1;
    }
    return 0;
}

// Expect a token or exit with error
static void expect(VexTokenType type)
{
    if (!match(type))
    {
        printf("Parse error: expected %d, got %d\n", type, cur_token.type);
        exit(1);
    }
}

// Forward declarations check ast.h to see or add your logic and custom funcs
AstNode *parse_statement();
AstNode *parse_block();
AstNode *parse_expression();
AstNode *parse_var_decl();
AstNode *parse_if();
AstNode *parse_while();
AstNode *parse_do();
AstNode *parse_func_decl();
AstNode *parse_func_call();

// Entry point(start)
AstNode *parse_program()
{
    AstNodeList *stmts = NULL, *last = NULL;
    advance();
    while (cur_token.type != TOK_EOF)
    {
        AstNode *stmt = parse_statement();
        AstNodeList *item = malloc(sizeof(AstNodeList));
        item->node = stmt;
        item->next = NULL;
        if (!stmts)
            stmts = last = item;
        else
        {
            last->next = item;
            last = item;
        }
    }

    AstNode *prog = malloc(sizeof(AstNode));
    prog->type = AST_PROGRAM;
    prog->program.stmts = stmts;
    return prog;
}

// Statement parser
AstNode *parse_statement()
{
    if (cur_token.type == TOK_BIND || cur_token.type == TOK_BINDM)
        return parse_var_decl();
    else if (cur_token.type == TOK_FUNC)
        return parse_func_decl();
    else if (cur_token.type == TOK_IF)
        return parse_if();
    else if (cur_token.type == TOK_WHILE)
        return parse_while();
    else if (cur_token.type == TOK_DO)
        return parse_do();
    else if (cur_token.type == TOK_LBRACE)
        return parse_block();
    else
        return parse_expression();
}

// Parse a block { ... } (has do{}support but also works without do lol yh i know i am flexing)
AstNode *parse_block()
{
    expect(TOK_LBRACE);
    AstNodeList *stmts = NULL, *last = NULL;
    while (cur_token.type != TOK_RBRACE && cur_token.type != TOK_EOF)
    {
        AstNode *stmt = parse_statement();
        AstNodeList *item = malloc(sizeof(AstNodeList));
        item->node = stmt;
        item->next = NULL;
        if (!stmts)
            stmts = last = item;
        else
        {
            last->next = item;
            last = item;
        }
    }
    expect(TOK_RBRACE);
    AstNode *block = malloc(sizeof(AstNode));
    block->type = AST_BLOCK;
    block->block.stmts = stmts;
    return block;
}

// Variable declaration (bind/bindm)(against my will but meh)
AstNode *parse_var_decl()
{
    int is_mut = (cur_token.type == TOK_BINDM);
    advance();
    if (cur_token.type != TOK_IDENTIFIER)
    {
        printf("Parse error: expected identifier after bind/bindm\n");
        exit(1);
    }
    char name[64];
    strcpy(name, cur_token.text);
    advance();

    // Handle type annotation: "as type" (Vel) or ": type" (Vex)
    AstNode *type_node = NULL;
    if (cur_token.type == TOK_AS) // Vel syntax: "as type"
    {
        advance(); // consume "as"
        if (cur_token.type == TOK_TYPE || cur_token.type == TOK_IDENTIFIER) // type name like "int" or "i32"
        {
            type_node = malloc(sizeof(AstNode));
            type_node->type = AST_IDENTIFIER;
            strcpy(type_node->identifier.name, cur_token.text);
            advance();
        }
    }
    else if (cur_token.type == TOK_TYPE) // Vex syntax: ": type"
    {
        advance(); // consume ":"
        if (cur_token.type == TOK_TYPE || cur_token.type == TOK_IDENTIFIER) // type name like "int" or "i32"
        {
            type_node = malloc(sizeof(AstNode));
            type_node->type = AST_IDENTIFIER;
            strcpy(type_node->identifier.name, cur_token.text);
            advance();
        }
    }

    // Handle assignment: "=" or ":="
    if (cur_token.type == TOK_COLON_EQ) // Vex syntax: ":="
    {
        advance(); // consume ":="
    }
    else
    {
        expect(TOK_ASSIGN); // "="
    }

    AstNode *value = parse_expression();

    AstNode *decl = malloc(sizeof(AstNode));
    decl->type = AST_VAR_DECL;
    strcpy(decl->var_decl.name, name);
    decl->var_decl.is_mut = is_mut;
    decl->var_decl.type = type_node;
    decl->var_decl.value = value;
    return decl;
}

// If statement
AstNode *parse_if()
{
    advance();
    AstNode *cond = parse_expression();
    AstNode *then_block = parse_block();
    AstNode *else_block = NULL;
    if (match(TOK_ELSE))
    {
        else_block = parse_block();
    }
    AstNode *node = malloc(sizeof(AstNode));
    node->type = AST_IF;
    node->if_stmt.cond = cond;
    node->if_stmt.then_block = then_block;
    node->if_stmt.else_block = else_block;
    return node;
}

// While loop
AstNode *parse_while()
{
    advance();
    AstNode *cond = parse_expression();
    AstNode *body = parse_block();
    AstNode *node = malloc(sizeof(AstNode));
    node->type = AST_WHILE;
    node->while_stmt.cond = cond;
    node->while_stmt.body = body;
    return node;
}

// Do block (single block, like: do { ... })
AstNode *parse_do()
{
    advance();
    AstNode *body = parse_block();
    AstNode *node = malloc(sizeof(AstNode));
    node->type = AST_DO;
    node->do_stmt.body = body;
    return node;
}

// Function declaration (fn name(...) { ... })
AstNode *parse_func_decl()
{
    advance();
    if (cur_token.type != TOK_IDENTIFIER)
    {
        printf("Parse error: expected identifier after fn\n");
        exit(1);
    }
    char name[64];
    strcpy(name, cur_token.text);
    advance();

    expect(TOK_LPAREN);
    // TODO: parse parameters if needed
    expect(TOK_RPAREN);

    AstNode *body = parse_block();
    AstNode *fn = malloc(sizeof(AstNode));
    fn->type = AST_FUNC_DECL;
    strcpy(fn->func_decl.name, name);
    fn->func_decl.params = NULL; // TODO: params later
    fn->func_decl.body = body;
    return fn;
}

// Parse a simple expression (no operator precedence for now)
AstNode *parse_simple_expression()
{
    if (cur_token.type == TOK_NUMBER)
    {
        AstNode *node = malloc(sizeof(AstNode));
        node->type = AST_LITERAL;
        node->literal.int_val = cur_token.int_value;
        advance();
        
        // Handle type annotation: number@type (Vel syntax)
        if (cur_token.type == TOK_AT)
        {
            advance(); // consume "@"
            if (cur_token.type == TOK_IDENTIFIER || cur_token.type == TOK_TYPE)
            {
                // Create a TypeCast node
                AstNode *type_cast = malloc(sizeof(AstNode));
                type_cast->type = AST_TYPE_CAST;
                type_cast->type_cast.expr = node;
                strcpy(type_cast->type_cast.target_type, cur_token.text);
                advance();
                return type_cast;
            }
            else
            {
                printf("Parse error: expected type identifier after @\n");
                exit(1);
            }
        }
        
        return node;
    }
    else if (cur_token.type == TOK_STRING)
    {
        AstNode *node = malloc(sizeof(AstNode));
        node->type = AST_LITERAL;
        strcpy(node->literal.str_val, cur_token.text);
        advance();
        return node;
    }
    else if (cur_token.type == TOK_IDENTIFIER)
    {
        char name[64];
        strcpy(name, cur_token.text);
        advance();

        // Function call: identifier(...)
        if (cur_token.type == TOK_LPAREN)
        {
            advance();
            
            // Parse arguments
            AstNodeList *args = NULL;
            if (cur_token.type != TOK_RPAREN) {
                // Parse first argument
                AstNode *arg = parse_expression();
                args = malloc(sizeof(AstNodeList));
                args->node = arg;
                args->next = NULL;
                
                // Parse additional arguments
                while (cur_token.type == TOK_COMMA) {
                    advance(); // consume comma
                    AstNode *next_arg = parse_expression();
                    AstNodeList *new_item = malloc(sizeof(AstNodeList));
                    new_item->node = next_arg;
                    new_item->next = NULL;
                    
                    // Add to end of list
                    AstNodeList *current = args;
                    while (current->next) current = current->next;
                    current->next = new_item;
                }
            }
            
            expect(TOK_RPAREN);
            AstNode *node = malloc(sizeof(AstNode));
            node->type = AST_FUNC_CALL;
            strcpy(node->func_call.name, name);
            node->func_call.args = args;
            return node;
        }
        // Plain identifier
        AstNode *node = malloc(sizeof(AstNode));
        node->type = AST_IDENTIFIER;
        strcpy(node->identifier.name, name);
        return node;
    }
    else
    {
        printf("Parse error: unexpected token in expression (token: '%s', type: %d)\n", cur_token.text, cur_token.type);
        exit(1);
    }
}

// Expression parser with binary operations
AstNode *parse_expression()
{
    AstNode *left = parse_simple_expression();
    
    // Check for binary operators
    while (cur_token.type == TOK_PLUS || cur_token.type == TOK_MINUS || 
           cur_token.type == TOK_STAR || cur_token.type == TOK_SLASH ||
           cur_token.type == TOK_LT || cur_token.type == TOK_GT ||
           cur_token.type == TOK_EQ || cur_token.type == TOK_NE ||
           cur_token.type == TOK_AND || cur_token.type == TOK_OR ||
           cur_token.type == TOK_EXCLAM || cur_token.type == TOK_ASSIGN) // nullish coalescing and assignment
    {
        // Handle assignment specially
        if (cur_token.type == TOK_ASSIGN) {
            advance(); // consume operator
            if (left->type != AST_IDENTIFIER) {
                printf("Parse error: left side of assignment must be an identifier\n");
                exit(1);
            }
            AstNode *right = parse_expression(); // Assignment is right-associative
            AstNode *assign = malloc(sizeof(AstNode));
            assign->type = AST_ASSIGN;
            strcpy(assign->var_decl.name, left->identifier.name);
            assign->var_decl.value = right;
            left = assign;
        } else {
            char op[4];
            if (cur_token.type == TOK_PLUS) strcpy(op, "+");
            else if (cur_token.type == TOK_MINUS) strcpy(op, "-");
            else if (cur_token.type == TOK_STAR) strcpy(op, "*");
            else if (cur_token.type == TOK_SLASH) strcpy(op, "/");
            else if (cur_token.type == TOK_LT) strcpy(op, "<");
            else if (cur_token.type == TOK_GT) strcpy(op, ">");
            else if (cur_token.type == TOK_EQ) strcpy(op, "==");
            else if (cur_token.type == TOK_NE) strcpy(op, "!=");
            else if (cur_token.type == TOK_AND) strcpy(op, "&&");
            else if (cur_token.type == TOK_OR) strcpy(op, "||");
            else if (cur_token.type == TOK_EXCLAM) strcpy(op, "!");
            
            advance(); // consume operator
            AstNode *right = parse_simple_expression();
            
            // Create binary operation node
            AstNode *bin_op = malloc(sizeof(AstNode));
            bin_op->type = AST_BIN_OP;
            bin_op->bin_op.left = left;
            bin_op->bin_op.right = right;
            strcpy(bin_op->bin_op.op, op);
            
            left = bin_op;
        }
    }
    
    // Handle type casting at expression level: expression@type
    if (cur_token.type == TOK_AT)
    {
        advance(); // consume "@"
        if (cur_token.type == TOK_IDENTIFIER || cur_token.type == TOK_TYPE)
        {
            // Create a TypeCast node
            AstNode *type_cast = malloc(sizeof(AstNode));
            type_cast->type = AST_TYPE_CAST;
            type_cast->type_cast.expr = left;
            strcpy(type_cast->type_cast.target_type, cur_token.text);
            advance();
            return type_cast;
        }
        else
        {
            printf("Parse error: expected type identifier after @\n");
            exit(1);
        }
    }
    
    return left;
}

