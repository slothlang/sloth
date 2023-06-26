#include <unistd.h>
#include <stdlib.h>
#include <string.h>

void wait(long long x) {
    sleep(x);
}

long long slen(char *str) {
    return (long long) strlen(str);
}

char charAt(char *str, long long x) {
    return str[x];
}

long long parse_int(char *str) {
    return (long long) atoi(str);
}
