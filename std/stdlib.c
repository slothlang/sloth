#include <unistd.h>
#include <stdlib.h>
#include <string.h>

void wait(int x) {
    sleep(x);
}

int slen(char *str) {
    return (int) strlen(str);
}

char charAt(char *str, int x) {
    return str[x];
}

int parse_int(char *str) {
    return (int) atoi(str);
}
