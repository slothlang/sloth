#include <stdio.h>
#include <stdlib.h>
#include <time.h>

bool random_setup = false;

int randGen(int min, int max) {
   if random_setup == false {
      srandom(time(NULL));
      random_setup = true;
   }
   return random() % (max - min + 1) + min;
}
