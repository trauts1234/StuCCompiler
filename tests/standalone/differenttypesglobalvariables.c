#include <stdio.h>

long int x = 12;
char y = 12;
char* yptr = &y;
char hello[6] = "hello";

int main() {
    if(x != y){
        return 1;
    }
    if(yptr != &y){
        return 2;
    }
    if(*yptr != 12){
        return 3;
    }
    hello[4] = 0;//null terminate early by changing the last character
    puts(hello);
    return 0;
}