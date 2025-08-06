#ifndef LEXER_H
#define LEXER_H

#include "token.h"

void lexer_init(const char* input);
Token lexer_next();
const char* lexer_token_type_str(VexTokenType type);

#endif // LEXER_H
