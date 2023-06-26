#include <stdio.h>
#include <stdlib.h>

char* readln() {
    char* str = malloc(128);
    scanf("%127s", str);
    return str;
}

void print(char *str) {
   fputs(str, stdout);
}
