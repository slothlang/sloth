#include <stdio.h>
#include <stdlib.h>
#include <time.h>


long long randGen(long long min, long long max) {
   srandom((unsigned) time(NULL)); 
   return random() % (max - min + 1) + min;
}
