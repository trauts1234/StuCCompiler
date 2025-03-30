struct x {int a;int b;};

int main() {
    struct x foo;
    foo.a = 69;
    foo.b = 0;

    int *aptr = &foo.a, *bptr = &foo.b;

    if(foo.a != *aptr){
        return 1;
    }
    if(foo.b != *bptr){
        return 2;
    }
    return 0;
}