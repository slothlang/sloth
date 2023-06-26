#include <stdio.h>

typedef struct {
    int size;
    int cap;
    long* inner;
} IntVec;

IntVec* test();

int main() {
    IntVec* v = test();

    int size = (*v).size;
    int cap = (*v).cap;
    long* inner = (*v).inner;

    printf("%d\n", size);
    printf("%d\n", cap);

    for (int i = 0; i < size; ++i) {
        long value = inner[i];
        printf("%d ", i);
    }
    puts("\n");
}
