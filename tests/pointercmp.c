int main() {

    int arr[2];


    if(&arr[0] < &arr[1]) {//compare pointers (first should be smaller)
        return 0;
    }

    return 1;
}