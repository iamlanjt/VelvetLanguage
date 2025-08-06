#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "parser.h"
#include "eval.h"
#include "lexer.h"
#include "codegen.h"

int main(int argc, char* argv[]) 
{
    if (argc == 2) 
    {
        // Command line argument provided
        if (strcmp(argv[1], "--help") == 0 || strcmp(argv[1], "-h") == 0) 
        {
            print_usage(argv[0]);
            return 0;
        }
        
        // Check if it's a .vex or .vel file
        if (strstr(argv[1], ".vex") != NULL || strstr(argv[1], ".vel") != NULL) {
            // Read and execute .vex file
            FILE* file = fopen(argv[1], "r");
            if (!file) {
                printf("Error: Cannot open file '%s'\n", argv[1]);
                return 1;
            }
            
            // Read file content
            char input[4096];
            size_t bytes_read = fread(input, 1, sizeof(input) - 1, file);
            input[bytes_read] = '\0';
            fclose(file);
            
            lexer_init(input);
            
            // Debug: print first few tokens
            printf("Debug: First 10 tokens:\n");
            Token t;
            for (int i = 0; i < 10; i++) {
                t = lexer_next();
                printf("Token %d: '%s' (type=%d)\n", i, t.text, t.type);
                if (t.type == TOK_EOF) break;
            }
            
            lexer_init(input);
            AstNode* root = parse_program();
            eval_program(root);
            return 0;
        }
        
        create_vexl_project(argv[1]);
    } else if (argc == 1) {
        // No arguments, use interactive mode bruhhh
        init_interactive();
    } else {
        // Too many arguments bro this is a new lang dont make work hard for us
        printf("Too many arguments.\n");
        print_usage(argv[0]);
        return 1;
    }
    return 0;
}