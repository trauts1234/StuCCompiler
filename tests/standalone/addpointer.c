int main() {
    int x=1;
    int y=-1;

    int* ptr = &y;

    return *(1 + ptr);//this should go up the stack by 1 and read the value of x
}