#include <errno.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <stdio.h>

int wait(int msec) {
    struct timespec ts;
    int res;

    if (msec < 0)
    {
        errno = EINVAL;
        return -1;
    }

    ts.tv_sec = msec / 1000;
    ts.tv_nsec = (msec % 1000) * 1000000;

    do {
        res = nanosleep(&ts, &ts);
    } while (res && errno == EINTR);

    return res;
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

char* istr(int x) {
    char* snum[100];
    sprintf(snum, "%d", x);
    //char* result = snum;
    return snum;
}
