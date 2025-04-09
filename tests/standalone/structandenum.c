struct x {int a;};

enum y {a,b};

int main() {
    struct x foo;
    foo.a = 0;//ensure that ". a" is not replaced by an enum value

    return foo.a;
}