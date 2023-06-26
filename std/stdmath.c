#include <stdio.h>
#include <stdlib.h>
#include <time.h>

int randGen(int min, int max) {
   time_t t;
   
   srand((unsigned) time(&t));
   
   return rand() % (max - min + 1) + min;
}
