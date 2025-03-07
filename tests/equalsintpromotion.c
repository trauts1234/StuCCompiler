int main() {
    int x= 2147483647;
    int y = x;

    long long int z = x+y;//x and y stay as int in this calculation, so the overflow means that z=0
    return z / 2147483647;
}