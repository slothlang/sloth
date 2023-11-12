#include <stdio.h>
#include <stdlib.h>

char *readln() {
  char *str = malloc(128);
  fgets(str, 127, stdin);
  return str;
}

void print(char *str) { fputs(str, stdout); }

void termpos(int x, int y) { printf("\x1b[%d;%dH", x, y); }

void termclear() { printf("\x1b[2J\x1b[H"); }

void curshide() { print("\x1b[?25l"); }

void cursshow() { print("\x1b[?25h"); }

char *filer(char *path) {
  FILE *fptr = fopen(path, "rb");
  char *contents = 0;

  if (fptr == NULL) {
    return "File not found";
  }
  fseek(fptr, 0, SEEK_END);
  long size = ftell(fptr);
  fseek(fptr, 0, SEEK_SET);

  contents = malloc(size);
  fread(contents, 1, size, fptr);
  fclose(fptr);

  return contents;
}
