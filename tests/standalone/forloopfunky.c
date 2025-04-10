#include <stdio.h>
int main() {
    for(;;){
        printf("A");
        break;
    }

    int x = 1;

    for(;x<5;++x){
        printf("%d", x);
    }

    for(int i=0;;){
        printf("B");
        break;
    }

    return 0;
}