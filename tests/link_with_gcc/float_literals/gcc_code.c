extern float a;
extern float b;
extern float c;
extern float d;
extern float e;
extern float f;
extern float g;
extern float h;
extern float i;

int test_global() {
    if(a != 0) {
        return 1;
    }
    if(b != 1) {
        return 2;
    }
    if(c != -123.456) {
        return 3;
    }
    if(d != 1e10) {
        return 4;
    }
    if(e != 1e-10f) {
        return 5;
    }
    if(f != 3.14159265) {
        return 6;
    }
    if(g != .5) {
        return 7;
    }
    if(h != 5.0) {
        return 8;
    }
    if(i != 0x1.1p+2f) {
        return 9;
    }

    return 0;
}