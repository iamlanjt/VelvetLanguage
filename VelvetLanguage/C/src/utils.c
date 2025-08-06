#include "utils.h"
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdarg.h>

// Error handling function
void die(const char* msg) {
    fprintf(stderr, "Error: %s\n", msg);
    exit(1);
}

// Formatted error function
void dief(const char* format, ...) {
    va_list args;
    va_start(args, format);
    fprintf(stderr, "Error: ");
    vfprintf(stderr, format, args);
    fprintf(stderr, "\n");
    va_end(args);
    exit(1);
}

// Warning function
void warn(const char* msg) {
    fprintf(stderr, "Warning: %s\n", msg);
}

// Formatted warning function
void warnf(const char* format, ...) {
    va_list args;
    va_start(args, format);
    fprintf(stderr, "Warning: ");
    vfprintf(stderr, format, args);
    fprintf(stderr, "\n");
    va_end(args);
}

// Safe memory allocation
void* safe_malloc(size_t size) {
    void* ptr = malloc(size);
    if (!ptr) {
        die("Memory allocation failed");
    }
    return ptr;
}

// Safe memory reallocation
void* safe_realloc(void* ptr, size_t size) {
    void* new_ptr = realloc(ptr, size);
    if (!new_ptr) {
        die("Memory reallocation failed");
    }
    return new_ptr;
}

// Safe string duplication
char* safe_strdup(const char* str) {
    if (!str) return NULL;
    char* dup = safe_malloc(strlen(str) + 1);
    strcpy(dup, str);
    return dup;
}

// Check if string is empty or whitespace only
int is_empty_or_whitespace(const char* str) {
    if (!str) return 1;
    while (*str) {
        if (*str != ' ' && *str != '\t' && *str != '\n' && *str != '\r') {
            return 0;
        }
        str++;
    }
    return 1;
}

// Trim whitespace from string
char* trim_whitespace(char* str) {
    if (!str) return NULL;
    
    // Trim leading whitespace
    while (*str && (*str == ' ' || *str == '\t' || *str == '\n' || *str == '\r')) {
        str++;
    }
    
    if (*str == '\0') return str;
    
    // Trim trailing whitespace
    char* end = str + strlen(str) - 1;
    while (end > str && (*end == ' ' || *end == '\t' || *end == '\n' || *end == '\r')) {
        end--;
    }
    end[1] = '\0';
    
    return str;
}