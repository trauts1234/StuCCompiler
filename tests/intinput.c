#include <stdio.h>

int main(int argc, char** argv) {//for some reason this only seems to crash when argc and argv are used
    for(int i=0;i<3;i = i+1) {
        int option=0;
        scanf("%d", &option);
        puts("parsed text");
    }

    return 0;
}