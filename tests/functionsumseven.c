int sumseven(int a, long long int b, int c, int d, int e, int f, int g) {
    return a + b + c + d + e + f + g;
}

int main() {
    int x = 1;
    int result = sumseven(x,x*2,x*3,x*3 + 1,5,6, 7);

    return result;
}