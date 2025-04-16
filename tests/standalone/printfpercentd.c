#include <stdio.h>
int main() {
    printf("");//print blank string to trip up compiler
    int result = printf("score: %d", 12);
    return result;//returns the number of characters printed
}