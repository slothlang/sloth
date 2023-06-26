#include <stdio.h>

typedef struct {
    int size;
    int cap;
    int* inner;
} IntVec;

IntVec* test();

int main() {
    IntVec* v = test();

    int size = (*v).size;
    int cap = (*v).cap;
    int* inner = (*v).inner;

    printf("%d\n", size);
    printf("%d\n", cap);

    for (int i = 0; i < size; ++i) {
        int value = inner[i];
        printf("%d ", value);
    }
    puts("\n");
}
