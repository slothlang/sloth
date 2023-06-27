#include <stdio.h>

void print(char* x) {
    printf("%s", x);
}
void printfl(float x) {
    printf("%f", x);
}
void printint(int x) {
    printf("%d", x);
}

int as_int(float x) {
    return (int) x;
}

void termpos(int x, int y) {
    printf("\x1b[%d;%dH", x, y);
}
