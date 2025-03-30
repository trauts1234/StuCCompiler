#include<stdio.h>

void print_a_(int times) {
    if (times == 0){
        return;
    }
    printf("a");
    print_a_(times-1);
}

int main() {
    print_a_(7);
}