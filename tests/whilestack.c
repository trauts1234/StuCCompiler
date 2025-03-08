#include<stdio.h>
int main() {
    int sum=0, i = 0;
    while(i<3) {
        int x = i;
        sum = sum + x;
        puts("loop");

        i = i+1;
    }
    return sum;
}