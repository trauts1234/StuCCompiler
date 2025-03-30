int main() {
    enum x {A,B} foo = A, *bar = &foo;

    if(*bar != foo){
        return 1;
    }

    if (foo != A){
        return 2;
    }

    return 0;
}