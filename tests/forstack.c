#include<stdio.h>
int main() {
    int sum = 0;
    for(int i=0;i<3;i = i+1) {
        int x = i;
        sum = sum + x;
        puts("loop");
    }
    return sum;
}