#include <stdio.h>
int main() {
    unsigned char x = 255;

    long long y = ++x;//this should overflow *first*, then type widened to 64 bit

    printf("%lld\n", y);
}