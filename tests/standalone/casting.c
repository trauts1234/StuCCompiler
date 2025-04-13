int test_integer_casting() {

    int x = 42;
    long y = (long)x;
    if (!(y == 42)) { return 1; };


    int large_val = 0x7fffffff;
    short s = (short)large_val;
    if (!(s == (short)0x7fffffff)) { return 1; };


    int neg = -5;
    unsigned int u = (unsigned int)neg;
    if (!(u == (unsigned int)-5)) { return 1; };


    int zero = 0;
    long long_zero = (long)zero;
    if (!(long_zero == 0)) { return 1; };


    unsigned int uint_max = 0xfffffffffff;
    int int_cast = (int)uint_max;
    if (!(int_cast == -1)) { return 1; };


    int a = 5, b = 3;
    long result = (long)(a + b);
    if (!(result == 8)) { return 1; };

    return 0;
}


int test_pointer_casting() {

    int x = 42;
    int* p_int = &x;
    void* p_void = (void*)p_int;
    int* p_back = (int*)p_void;
    if (!(p_back == p_int)) { return 1; };
    if (!(*p_back == 42)) { return 1; };


    int arr[5];
    arr[0] = 1; arr[1] = 2; arr[2] = 3; arr[3] = 4; arr[4] = 5;
    int* p_arr = arr;
    char* p_char = (char*)p_arr;
    if (!((void*)p_char == (void*)arr)) { return 1; };


    unsigned char* bytes = (unsigned char*)arr;


    if (!(bytes[0] == 1 || bytes[0] == 0)) { return 1; };


    int y = 100;
    int* py = &y;
    int** ppy = &py;
    void** ppv = (void**)ppy;
    int** ppy_back = (int**)ppv;
    if (!(ppy_back == ppy)) { return 1; };
    if (!(*ppy_back == py)) { return 1; };
    if (!(**ppy_back == 100)) { return 1; };



    int z = 255;
    int* pz = &z;
    char* pc = (char*)pz;
    short* ps = (short*)pc;
    int* pz_back = (int*)ps;
    if (!(pz_back == pz)) { return 1; };

    return 0;
}


int test_struct_cast() {

    struct A {
        int x;
        int y;
    };

    struct B {
        int a;
        int b;
    };

    struct A sa;
    sa.x = 1;
    sa.y = 2;
    struct B* sb = (struct B*)&sa;
    if (!(sb->a == 1)) { return 1; };
    if (!(sb->b == 2)) { return 1; };


    struct SingleInt {
        int value;
    };

    struct SingleInt si;
    si.value=42;
    int* pi_struct = (int*)&si;
    if (!(*pi_struct == 42)) { return 1; };

    return 0;
}


int test_enum_casting() {

    enum Color { RED, GREEN, BLUE };
    enum Color c = GREEN;
    int i_enum = (int)c;
    if (!(i_enum == 1)) { return 1; };


    int val = 2;
    enum Color back = (enum Color)val;
    if (!(back == BLUE)) { return 1; };


    int invalid = 999;
    enum Color invalid_color = (enum Color)invalid;
    if (!((int)invalid_color == 999)) { return 1; };


    int weird = (enum { X, Y, Z })Y;
    if (!(weird == 1)) { return 1; };


    int val2 = (enum { A = 5, B = 10, C = 15 })B;
    if (!(val2 == 10)) { return 1; };


    enum Signs { NEGATIVE = -1, ZERO = 0, POSITIVE = 1 };
    unsigned int u_sign = (unsigned int)NEGATIVE;
    if (!(u_sign == (unsigned int)-1)) { return 1; };


    enum SmallVals { SMALL1 = 1, SMALL2 = 2 };
    void* ptr = (void*)(unsigned long long)SMALL2;
    enum SmallVals back_val = (enum SmallVals)(unsigned long long)ptr;
    if (!(back_val == SMALL2)) { return 1; };

    return 0;
}


int test_compound_casts() {

    int weird_struct_val = (struct { int x; int y; }){.x = 5, .y = 10}.y;
    if (!(weird_struct_val == 10)) { return 1; };



    int arr[5];
    arr[0] = 1; arr[1] = 2; arr[2] = 3; arr[3] = 4; arr[4] = 5;
    short* short_arr = (short*)arr;


    if (!(short_arr[0] == 1 || short_arr[1] == 0 || short_arr[0] == 0)) { return 1; };


    int comma_val = (int)(1, 2, 3);
    if (!(comma_val == 3)) { return 1; };


    int base = 65;
    char c1 = (char)base;
    int back = (int)(char)(unsigned char)(signed char)c1;
    if (!(back == 65)) { return 1; };

    return 0;
}


int main() {

    int result;

    result = test_integer_casting();
    if (result != 0) return 1;

    result = test_pointer_casting();
    if (result != 0) return 2;

    result = test_struct_cast();
    if (result != 0) return 3;

    result = test_enum_casting();
    if (result != 0) return 4;

    result = test_compound_casts();
    if (result != 0) return 5;


    return 0;
}
