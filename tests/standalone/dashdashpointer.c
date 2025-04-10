int main() {
    int arr[3];
    arr[0] = 0;
    arr[1] = 1;
    arr[2] = 2;

    int *ptr = &arr[2];

    int* ptr2 = --ptr;

    if(*ptr !=1){
        return 1;
    }
    if(*ptr2 != 1) {
        return 2;
    }

    return 0;
}