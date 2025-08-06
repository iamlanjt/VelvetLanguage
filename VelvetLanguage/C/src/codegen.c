#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <io.h>
#include <direct.h>
#include <limits.h>
#ifndef MAX_PATH
#define MAX_PATH 260
#endif
#include <ctype.h>
#include "ast.h"
#include "parser.h"
#include "typecheck.h"
#include "token.h"
#define CF_DEBUG_MODE 0
#include "codegen.h"
#define MAX_VARIABLES 64

VarInfo variables[MAX_VARIABLES];
int variable_count = 0;

int add_variable(const char* name, ValueType ty, void* value, int is_mutable)
{
    if (variable_count >= MAX_VARIABLES)
        return -1;
    strncpy(variables[variable_count].name, name, 31);
    variables[variable_count].name[31] = 0;
    variables[variable_count].ty = ty;
    variables[variable_count].value = value;
    variables[variable_count].is_mutable = is_mutable;
    variable_count++;
    return variable_count-1;
}


// Function to validate project name basically if someone does w as name or edfgfdfcvgfdfgvhfdfghfdfghfdgh as name
int is_valid_project_name(const char *name) {
    if (strlen(name) == 0 || strlen(name) > 40) {
        return 0; // Too short or too long
    }
    
    // Check for invalid characters the stupid guys or people who make misteales forgive the spelling am about the coding
    for (int i = 0; name[i] != '\0'; i++) {
        char c = name[i];
        if (!isalnum(c) && c != '_' && c != '-') {
            return 0; // Invalid character bro think
        }
    }
    
    // Can't start with a number
    if (isdigit(name[0])) {
        return 0;
    }
    
    return 1; // Valid type shi
}

void create_vexl_project(const char *name) {
    // Validate project name first cause why not
    if (!is_valid_project_name(name)) {
        printf("   Invalid project name! Use only letters, numbers, underscores, and hyphens.\n");
        printf("   Name must be 1-40 characters and not start with a number.\n");
        return;
    }

    char path[MAX_PATH];
    snprintf(path, MAX_PATH, ".\\%s", name);

    // Check if project already exists
    if (_access(path, 0) == 0) {
        printf("Project '%s' already exists!\n", name);
        return;
    }

    // Create project directory
    if (_mkdir(path) != 0) {
        perror("Failed to create project directory");
        return;
    }
    printf("Created project directory: %s\n", name);

    // Create src directory
    char srcPath[MAX_PATH];
    snprintf(srcPath, MAX_PATH, "%s\\src", path);
    if (_mkdir(srcPath) != 0) {
        perror("Failed to create src directory");
        return;
    }
    printf("Created src directory\n");

    // Create main.vex
    char mainVexPath[MAX_PATH];
    snprintf(mainVexPath, MAX_PATH, "%s\\main.vex", srcPath);
    FILE *mainVex = fopen(mainVexPath, "w");
    if (mainVex) {
        fprintf(mainVex, "// Source entry for %s\n", name);
        fprintf(mainVex, "fn main() {\n    println(\"Hello from main.vex!\");\n}\n");
        fclose(mainVex);
        printf("Created main.vex\n");
    } else {
        perror("Couldn't create main.vex");
        return;
    }

    // Create main.vel
    char mainVelPath[MAX_PATH];
    snprintf(mainVelPath, MAX_PATH, "%s\\main.vel", srcPath);
    FILE *mainVel = fopen(mainVelPath, "w");
    if (mainVel) {
        fprintf(mainVel, "// Logic layer for %s\n", name);
        fprintf(mainVel, "def main() {\n    echo \"Hello from main.vel\"\n}\n");
        fclose(mainVel);
        printf("Created main.vel\n");
    } else {
        perror("Couldn't create main.vel"); // sometimes some users are wierd like why tf do i need to handel errors
        return;
    }

    // Create config.vexl
    char configPath[MAX_PATH];
    snprintf(configPath, MAX_PATH, "%s\\config.vexl", path);
    FILE *configFile = fopen(configPath, "w");
    if (configFile) {
        fprintf(configFile,
            "[project]\n"
            "name = \"%s\"\n"
            "main_source = \"src\\main.vex\"\n"
            "main_logic = \"src\\main.vel\"\n"
            "version = \"0.1.0\"\n"
            "author = \"Void\"\n", name);
        fclose(configFile);
        printf("Created config.vexl\n");
    } else {
        perror("Couldn't create config.vexl"); //i dont know how this will ever fail but doing this for some wierd users sake
        return;
    }
    //they dont need this 
    //meh actually u can remove this to save space
    // but like we like to be nice no?
    printf("\n Velvet project '%s' initialized successfully!\n", name);
    printf("To get started:\n");
    printf("cd %s\n", name);
    printf("# Edit src\\main.vex and src\\main.vel\n");
}

void init_interactive() {
    char name[50];
    printf(" Velvet Project Initializer\n");
    printf("Enter project name: ");
    
    if (fgets(name, sizeof(name), stdin)) {
        // Remove newline if present(Reasons why c is da goat @velvet)
        size_t len = strlen(name);
        if (len > 0 && name[len-1] == '\n') {
            name[len-1] = '\0';
        }
        
        if (strlen(name) == 0) {
            printf("Project name cannot be empty.\n");
            return;
        }
        
        create_vexl_project(name);
    } else {
        printf("Failed to read input.\n"); //you somehow failed to put in the appropriate name
    }
}

void print_usage(const char *program_name) {
    printf("Usage: %s [project_name]\n", program_name);
    printf("  If no project name is provided, interactive mode will be used.\n");
    printf("\nExamples:\n");
    printf("  %s my_project    # Create project 'my_project'\n", program_name);
    printf("  %s               # Interactive mode\n", program_name);
}

