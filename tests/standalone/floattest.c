#include <stdio.h>
#include <math.h>
#include <float.h>
#include <stdlib.h>
#include <string.h>

int float_equals(float a, float b) {
    return fabsf(a - b) < 1e-6f;
}

int double_equals(double a, double b) {
    return fabs(a - b) < 1e-12;
}

struct FloatStruct {
    float x;
    float y;
};

int main() {
    // 1. Basic float/double literals
    float f1 = 1.5f, f2 = -2.25f;
    double d1 = 3.14159265358979, d2 = -0.5;

    if (!float_equals(f1 + f2, -0.75f)) return 1;;
    if (!double_equals(d1 + d2, 2.64159265358979)) return 1;;

    // 2. Arithmetic operators
    if (!float_equals(f1 * f2, -3.375f)) return 1;;
    if (!double_equals(d1 * d2, -1.570796326794895)) return 1;;

    if (!float_equals(f2 / f1, -1.5f)) return 1;;
    if (!double_equals(d2 / d1, -0.15915494309189535)) return 1;;

    // 3. Comparisons
    if (!(f1 > f2)) return 1;;
    if (!(d2 < d1)) return 1;;

    // 4. Prefix/postfix increment
    float fx = 1.0f;
    if (!float_equals(++fx, 2.0f)) return 1;;
    if (!float_equals(fx++, 2.0f)) return 1;;
    if (!float_equals(fx, 3.0f)) return 1;;

    // 5. Struct with floats
    struct FloatStruct s;
    s.x = 0.5f;
    s.y = 1.5f;
    if (!float_equals(s.x + s.y, 2.0f)) return 1;;

    // 6. Array of floats
    float arr[3] = { 0.1f, 0.2f, 0.3f };
    if (!float_equals(arr[0] + arr[1] + arr[2], 0.6f)) return 1;;

    // 7. Pointer casting and manipulation
    float *p = arr;
    p[1] = 0.4f;
    if (!float_equals(arr[1], 0.4f)) return 1;;

    // 8. Type casting and promotion
    double promoted = f1 + d1;
    if (!double_equals(promoted, 4.64159265358979)) return 1;;

    // 11. Infinity and NaN
    float inf = 1.0f / 0.0f;
    float ninf = -1.0f / 0.0f;
    float nanval = 0.0f / 0.0f;

    // 12. Subnormal values
    float subnormal = FLT_TRUE_MIN;
    if (subnormal == 0.0f) return 1;;

    // 13. memcpy round-trip
    float orig = 1.234567f;
    float copy;
    memcpy(&copy, &orig, sizeof(float));
    if (!float_equals(orig, copy)) return 1;;

    // 15. Division by non-zero
    if (!float_equals(1.0f / 2.0f, 0.5f)) return 1;;

    // 16. Compound assignment
    float a = 1.0f;
    a += 0.5f;
    if (!float_equals(a, 1.5f)) return 1;;

    double b = 2.0;
    b *= 3.0;
    if (!double_equals(b, 6.0)) return 1;;

    // 17. Casting between float and int
    float f_int = 7.5f;
    int i = (int)f_int;
    if (i != 7) return 1;;

    i = -3;
    f_int = (float)i;
    if (!float_equals(f_int, -3.0f)) return 1;;

    return 0;
}
