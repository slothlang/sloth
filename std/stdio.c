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

void termpos(int x, int y) {
    printf("\x1b[%d;%dH", x, y);
}

void termclear() {
    printf("\x1b[2J\x1b[H");
}

void curshide() {
    print("\x1b[?25l");
}

void cursshow() {
    print("\x1b[?25h");
}
