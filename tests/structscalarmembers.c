struct x {int a;int b;};

int main() {
    struct x foo;
    foo.a = 69;
    foo.b = 0;

    if(foo.a != 69){
        return 1;
    }
    if(foo.b != 0){
        return 2;
    }
    return 0;
}