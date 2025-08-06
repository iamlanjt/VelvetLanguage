#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <ctype.h>
#include "lexer.h"
#ifndef strtol
#warning "strtol is available"
#endif


static const char* src = NULL;
static int pos = 0;

void lexer_init(const char* input) 
{
    src = input;
    pos = 0;
}

static void skip_ws() 
{
    while (isspace(src[pos])) pos++;
}

static void skip_ws_and_comments() 
{
    while (1) 
    {
        while (isspace(src[pos])) pos++;
        // Skip '//' comment
        if (src[pos] == '/' && src[pos + 1] == '/') 
        {
            pos += 2;
            while (src[pos] && src[pos] != '\n') pos++;
        }
        // Skip '/*' comment
        else if (src[pos] == '/' && src[pos + 1] == '*') 
        {
            pos += 2;
            while (src[pos] && !(src[pos] == '*' && src[pos + 1] == '/')) pos++;
            if (src[pos] == '*') pos += 2;
        }
        // Skip ';;' comment (existing)
        else if (src[pos] == ';' && src[pos + 1] == ';') 
        {
            pos += 2;
            while (src[pos] && src[pos] != '\n') pos++;
        } 
        else 
        {
            break;
        }
    }
}

static int is_ident_start(char c) 
{
    return isalpha(c) || c == '_';
}
static int is_ident_char(char c) 
{
    return isalnum(c) || c == '_';
}

// Helper to check for multi-char tokens like :=
static int match(const char* str) 
{
    int len = strlen(str);
    if (strncmp(&src[pos], str, len) == 0) 
    {
        pos += len;
        return 1;
    }
    return 0;
}

int atoi(const char *);

// Main lexing function
Token lexer_next() 
{
    Token t = {0};
    skip_ws_and_comments();
    char c = src[pos];

    if (c == '\0') 
    {
        t.type = TOK_EOF;
        return t;
    }

    // Identifiers and keywords
    if (is_ident_start(c)) 
    {
        int start = pos;
        while (is_ident_char(src[pos])) pos++;
        int len = pos - start;
        if (len > 63) len = 63;
        strncpy(t.text, &src[start], len);
        t.text[len] = 0;

        // Keywords
        if (strcmp(t.text, "bind") == 0)      t.type = TOK_BIND;
        else if (strcmp(t.text, "bindm") == 0) t.type = TOK_BINDM;
        else if (strcmp(t.text, "fn") == 0)   t.type = TOK_FUNC;
        else if (strcmp(t.text, "if") == 0)   t.type = TOK_IF;
        else if (strcmp(t.text, "while") == 0) t.type = TOK_WHILE;
        else if (strcmp(t.text, "do") == 0)   t.type = TOK_DO;
        else if (strcmp(t.text, "else") == 0) t.type = TOK_ELSE;
        else if (strcmp(t.text, "as") == 0)   t.type = TOK_AS;
        else if (strcmp(t.text, "write") == 0) t.type = TOK_WRITE;
        else if (strcmp(t.text, "int") == 0 || strcmp(t.text, "i32") == 0 || strcmp(t.text, "i8") == 0) t.type = TOK_TYPE;
        else if (strcmp(t.text, "string") == 0 || strcmp(t.text, "str") == 0) t.type = TOK_TYPE;
        else if (strcmp(t.text, "float") == 0 || strcmp(t.text, "number") == 0) t.type = TOK_TYPE;
        else if (strcmp(t.text, "bool") == 0) t.type = TOK_TYPE;
        else if (strcmp(t.text, "any") == 0) t.type = TOK_TYPE;
        else t.type = TOK_IDENTIFIER;
        return t;
    }

    // Number literal
    if (isdigit(c)) 
    {
        int start = pos;
        while (isdigit(src[pos])) pos++;
        int len = pos - start;
        if (len > 63) len = 63;
        strncpy(t.text, &src[start], len);
        t.text[len] = 0;
        // Use long long for large numbers
        long long val = strtoll(t.text, NULL, 10);
        t.int_value = (int)val; // Truncate to int for now
        t.type = TOK_NUMBER;
        return t;
    }

    // String literal
    if (c == '"') 
    {
        pos++; // skip "
        int start = pos;
        while (src[pos] && src[pos] != '"') pos++;
        int len = pos - start;
        if (len > 63) len = 63;
        strncpy(t.text, &src[start], len);
        t.text[len] = 0;
        if (src[pos] == '"') pos++;
        t.type = TOK_STRING;
        return t;
    }

    // Symbols
    if (match(":=")) { t.type = TOK_COLON_EQ; strcpy(t.text, ":="); return t; }
    if (match("==")) { t.type = TOK_EQ; strcpy(t.text, "=="); return t; }
    if (match("!=")) { t.type = TOK_NE; strcpy(t.text, "!="); return t; }
    if (match("&&")) { t.type = TOK_AND; strcpy(t.text, "&&"); return t; }
    if (match("||")) { t.type = TOK_OR; strcpy(t.text, "||"); return t; }
    if (match("->")) { t.type = TOK_ARROW; strcpy(t.text, "->"); return t; }
    if (match("=>")) { t.type = TOK_FAT_ARROW; strcpy(t.text, "=>"); return t; }
    if (match("="))  { t.type = TOK_ASSIGN; strcpy(t.text, "=");  return t; }
    if (match(":"))  { t.type = TOK_TYPE;   strcpy(t.text, ":");  return t; }
    if (match("@"))  { t.type = TOK_AT;     strcpy(t.text, "@");  return t; }
    if (match("!"))  { t.type = TOK_EXCLAM; strcpy(t.text, "!");  return t; }
    if (match("["))  { t.type = TOK_LBRACKET; strcpy(t.text, "["); return t; }
    if (match("]"))  { t.type = TOK_RBRACKET; strcpy(t.text, "]"); return t; }
    if (match("."))  { t.type = TOK_DOT;    strcpy(t.text, ".");  return t; }
    if (match("{"))  { t.type = TOK_LBRACE; strcpy(t.text, "{");  return t; }
    if (match("}"))  { t.type = TOK_RBRACE; strcpy(t.text, "}");  return t; }
    if (match("("))  { t.type = TOK_LPAREN; strcpy(t.text, "(");  return t; }
    if (match(")"))  { t.type = TOK_RPAREN; strcpy(t.text, ")");  return t; }
    if (match(";"))  { t.type = TOK_SEMICOLON; strcpy(t.text, ";"); return t; }
    if (match(","))  { t.type = TOK_COMMA;  strcpy(t.text, ",");  return t; }
    if (match("+")) { t.type = TOK_PLUS; strcpy(t.text, "+"); return t; }
    if (match("-")) { t.type = TOK_MINUS; strcpy(t.text, "-"); return t; }
    if (match("*")) { t.type = TOK_STAR; strcpy(t.text, "*"); return t; }
    if (match("/")) { t.type = TOK_SLASH; strcpy(t.text, "/"); return t; }
    if (match("<")) { t.type = TOK_LT; strcpy(t.text, "<"); return t; }
    if (match(">")) { t.type = TOK_GT; strcpy(t.text, ">"); return t; }


    // Unknown
    t.type = TOK_EOF;
    pos++;
    return t;
}

// For debugging
const char* lexer_token_type_str(VexTokenType type) 
{
    switch(type) 
    {
        case TOK_EOF: return "EOF";
        case TOK_IDENTIFIER: return "IDENT";
        case TOK_NUMBER: return "NUMBER";
        case TOK_STRING: return "STRING";
        case TOK_BIND: return "BIND";
        case TOK_BINDM: return "BINDM";
        case TOK_FUNC: return "FUNC";
        case TOK_IF: return "IF";
        case TOK_WHILE: return "WHILE";
        case TOK_DO: return "DO";
        case TOK_TYPE: return "TYPE";
        case TOK_ASSIGN: return "ASSIGN";
        case TOK_LBRACE: return "LBRACE";
        case TOK_RBRACE: return "RBRACE";
        case TOK_LPAREN: return "LPAREN";
        case TOK_RPAREN: return "RPAREN";
        case TOK_SEMICOLON: return "SEMICOLON";
        case TOK_COMMA: return "COMMA";
        default: return "UNKNOWN";
    }

    //comments also whitespace
 

}