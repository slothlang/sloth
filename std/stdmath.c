#include <stdio.h>
#include <stdlib.h>
#include <time.h>

int random_setup = 0;

int randGen(int min, int max) {
   if (random_setup == 0) {
      srandom(time(NULL));
      random_setup = 1;
   }
   return random() % (max - min + 1) + min;
}
