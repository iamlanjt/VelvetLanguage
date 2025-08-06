#ifndef CODEGEN_H
#define CODEGEN_H

#include "ast.h"

typedef enum {
    TYPE_INT,
    TYPE_FLOAT,
    TYPE_STRING,
    TYPE_BOOL,
    TYPE_ARRAY_INT,
    
   
} ValueType;

typedef struct {
    char name[32];
    ValueType ty;
    void* value;
    int is_mutable;
} VarInfo;

int add_variable(const char* name, ValueType ty, void* value, int is_mutable);

// Project creation functions
void create_vexl_project(const char* name);
void init_interactive();
void print_usage(const char* program_name);

#endif
