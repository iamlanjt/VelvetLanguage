#ifndef TOKEN_H
#define TOKEN_H

typedef enum {
    TOK_EOF,
    TOK_IDENTIFIER,
    TOK_NUMBER,
    TOK_STRING,
    TOK_BIND,
    TOK_BINDM,
    TOK_ASSIGN,
    TOK_TYPE,
    TOK_FUNC,
    TOK_IF,
    TOK_WHILE,
    TOK_DO,
    TOK_LPAREN,
    TOK_RPAREN,
    TOK_LBRACE,
    TOK_RBRACE,
    TOK_COMMA,
    TOK_SEMICOLON,
    TOK_PLUS,
    TOK_MINUS,
    TOK_STAR,
    TOK_SLASH,
    TOK_LT,
    TOK_GT,
    TOK_ELSE,
    TOK_AT,           // @ for type annotations (Vel)
    TOK_COLON_EQ,     // := for type inference (Vex)
    TOK_AS,           // as for type declarations (Vel)
    TOK_ARROW,        // -> for function declarations (Vel)
    TOK_FAT_ARROW,    // => for function return type (Vel)
    TOK_EXCLAM,       // ! for nullish coalescing
    TOK_AND,          // && for logical AND
    TOK_OR,           // || for logical OR
    TOK_EQ,           // == for equality
    TOK_NE,           // != for inequality
    TOK_LBRACKET,     // [ for arrays
    TOK_RBRACKET,     // ] for arrays
    TOK_DOT,          // . for member access
    TOK_WRITE         // write function (Vex)

} VexTokenType;

typedef struct {
    VexTokenType type;
    char text[64];
    int int_value;
    //TODO add float_value, etc.
} Token;

#endif
