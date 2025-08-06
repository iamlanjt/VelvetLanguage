#ifndef AST_H
#define AST_H

typedef enum
{
    AST_PROGRAM,
    AST_BLOCK,
    AST_VAR_DECL,
    AST_ASSIGN,
    AST_FUNC_DECL,
    AST_FUNC_CALL,
    AST_IF,
    AST_WHILE,
    AST_DO,
    AST_LITERAL,
    AST_IDENTIFIER,
    AST_BIN_OP,
    AST_UN_OP,
    AST_TYPE_CAST
} AstNodeType;

typedef struct AstNode AstNode;
typedef struct AstNodeList AstNodeList;

struct AstNodeList
{
    AstNode *node;
    AstNodeList *next;
};

struct AstNode
{
    AstNodeType type;
    union
    {
        struct { AstNodeList *stmts; } program;
        struct { AstNodeList *stmts; } block;
        struct { char name[64]; int is_mut; AstNode *type; AstNode *value; } var_decl;
        struct { char name[64]; AstNodeList *params; AstNode *body; } func_decl;
        struct { char name[64]; AstNodeList *args; } func_call;
        struct { AstNode *cond; AstNode *then_block; AstNode *else_block; } if_stmt;
        struct { AstNode *cond; AstNode *body; } while_stmt;
        struct { AstNode *body; } do_stmt;
        struct { int int_val; double float_val; char str_val[128]; int bool_val; } literal;
        struct { char name[64]; } identifier;
        struct { AstNode *left; AstNode *right; char op[4]; } bin_op;
        struct { AstNode *expr; char op[4]; } un_op;
        struct { AstNode *expr; char target_type[16]; } type_cast;
    };
};

AstNode *parse_program();
void free_ast(AstNode *node);

#endif
