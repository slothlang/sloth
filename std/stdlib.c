#include <unistd.h>
#include <stdlib.h>
#include <string.h>

void wait(float x) {
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

int as_int(float x) {
    return (int) x;
}

// char* istr(int x) {
//     char snum[100];
//     return (char* )itoa(x, snum, 10);
// }
