#include<stdio.h>

int a = 1 - 2;
int b = 1u + 1;
unsigned int c = 1ull - 1ll;
long long int d = 2147483647 + 1;//integer, will overflow to -INT_MAX

int main(){
    if (a != -1){
        return 1;
    }
    if(b != 2) {
        return 2;
    }
    if(c != 0) {
        return 3;
    }
    if(d != -2147483648) {
        printf("%lld", d);
        return 4;
    }

    return 0;
}