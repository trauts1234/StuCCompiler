#include<stdio.h>

int main() {
    printf("hello world");

    int x=0;//sneaky variable declaration, that may mess up the stack for printf

    return 0;
}