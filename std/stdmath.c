#include <stdio.h>
#include <stdlib.h>
#include <time.h>


int randGen(int min, int max) {
   srandom((unsigned) time(NULL)); 
   return random() % (max - min + 1) + min;
}
