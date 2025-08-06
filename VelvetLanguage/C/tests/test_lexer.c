#include <stdlib.h>
#include <stdio.h>
int main() {
    printf("%ld\n", strtol("1337", NULL, 10));
    printf("%d\n", atoi("42"));
    return 0;
}
