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

int main () {
    test_global();
    float a = 0.0f;
    float b = 1.0f;
    float c = -123.456f;
    float d = 1e10f;
    float e = 1E-10f;
    float f = 3.14159265f;
    float g = .5f;
    float h = 5.f;
    float i = 0x1.1p+2f;  // Hex float (C99)
    return 0;
}
