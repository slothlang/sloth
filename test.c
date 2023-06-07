#include <stdio.h>

int add(int, int);
int subtract(int, int);

int main() {
    int a = add(5, 2);
    int b = subtract(3, 8);
    printf("%d %d\n", a, b);
    return 0;
}
