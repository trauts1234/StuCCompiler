#include<stdio.h>
int main() {
    long long int x = -1u;//the u means it gets cast to unsigned int then negated, so we get a big number

    printf("%lld", x);
}