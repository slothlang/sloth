#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
char *heap[100000];
int heaploc = 0;

int memalloc(int size) {
  const int chunk = heaploc;
  heap[heaploc++] = malloc(size);
  printf("MEMALLOC: heap[%d]\n", chunk);
  return chunk;
}
int drefi(int loc) {
  printf("DREF: *heap[%d] = %d\n", loc, *heap[loc]);
  return *heap[loc];
}
void assignrefi(int loc, int num) {
  *heap[loc] = num;
  printf("ASSREF: *heap[%d] = %d\n", loc, *heap[loc]);
}
