int main() {
    struct x {int a;int b;};

    struct x foo;

    foo.a = 0;
    foo.b = foo.a-1;

    if(foo.a != 0){
        return 1;
    }
    if(foo.b != -1) {
        return 2;
    }
    return 0;
}