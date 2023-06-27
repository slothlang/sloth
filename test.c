#include <stdio.h>
float pow(float, float);
int abs(int);
float fabs(float);
int max(int, int);
int min(int, int);
float fmax(float, float);
float fmin(float, float);

int main() {
    int x = pow(5, 3);
    printf("pow %d\n", x);
    int y = abs(-5);
    printf("abs %d\n", y);
    double z = fabs(-5.0);
    printf("fabs %f\n", z);
    int w = max(5, 3);
    printf("max %d\n", w);
    int n = min(5, 3);
    printf("min %d\n", n);
    int p = fmax(5.0, 3.0);
    printf("fmax %d\n", p);
    int g = fmin(5.0, 3.0);
    printf("fmin %d\n", g);
}
