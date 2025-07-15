float a = 0.0f;
float b = 1.0f;
float c = -123.456f;
float d = 1e10f;
float e = 1E-10f;
float f = 3.14159265f;
float g = .5f;
float h = 5.f;
float i = 0x1.1p+2f;  // Hex float (C99)

extern int test_global();
extern int test_local(float a, float b, float c, float d, float e, float f, float g, float h, float i);

int main () {
    int global_result = test_global();
    float a = 0.0f;
    float b = 1.0f;
    float c = -123.456f;
    float d = 1e10f;
    float e = 1E-10f;
    float f = 3.14159265f;
    float g = .5f;
    float h = 5.f;
    float i = 0x1.1p+2f;  // Hex float (C99)
    int local_result = test_local(a,b,c,d,e,f,g,h,i);

    if (global_result != 0) {
        return global_result;
    }
    if (local_result != 0) {
        return local_result + 100;
    }
    return 0;
}
