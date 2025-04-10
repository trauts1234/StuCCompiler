#include <stdio.h>
int main() {
    unsigned char x = 0;

    long long y = --x;//this should underflow *first*, then type widened to 64 bit

    printf("%lld\n", y);
}