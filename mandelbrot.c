#include <stdio.h>

int main() {
    float size = 800.0;
    float maxVal = 4.0;
    float maxIter = 50.0;
    float plane = 4.0;
    int x = 0;
    while (x < size) {
        int y = 0;
        while (y < size) {
            float cReal = (x * plane / size) - 2.0;
            float cImg = (y * plane / size) - 2.0;
            float zReal = 0.0;
            float zImg = 0.0;
            float count = 0.0;
            while ((zReal * zReal + zImg * zImg) <= maxVal && count < maxIter) {
                float temp = (zReal * zReal) - (zImg * zImg) + cReal;
                zImg = 2.0 * zReal * zImg + cImg;
                zReal = temp;
                count = count + 1;
                if (count == maxIter) {
                    printf("\x1b[%d;%dH", x, y);
                    printf("█");
                }
            }
            y = y + 1;
        }
        x = x + 1;
    }
    return 0;
}
