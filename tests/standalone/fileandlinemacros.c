#include<stdio.h>
int main() { if(__LINE__ != 2 ) {return 1;}
//    
/*

*/

if(__LINE__ != 8) {
    return 2;
}

printf(__FILE__);

#line 1000
if(__LINE__ != 1000) {
    return 3;
}

#line 10 "hello world.c"
if(__LINE__ != 10) {
    return 4;
}

printf(__FILE__);
}