double floor(double x) {
    int i = (int)x;
    if (x >= 0.0 || (double)i == x)
        return (double)i;
    else
        return (double)(i - 1);
}

double ceil(double x) {
    int i = (int)x;
    if (x <= 0.0 || (double)i == x)
        return (double)i;
    else
        return (double)(i + 1);
}

float floorf(float x) {
    int i = (int)x;
    if (x >= 0.0f || (float)i == x)
        return (float)i;
    else
        return (float)(i - 1);
}

float ceilf(float x) {
    int i = (int)x;
    if (x <= 0.0f || (float)i == x)
        return (float)i;
    else
        return (float)(i + 1);
}

double round(double x) {
    if (x >= 0.0) {
        return floor(x + 0.5);
    } else {
        return ceil(x - 0.5);
    }
}

float roundf(float x) {
    if (x >= 0.0f) {
        return floorf(x + 0.5f);
    } else {
        return ceilf(x - 0.5f);
    }
}

float fabsf(float __x) {
    if(__x > 0) {
        return __x;
    } else {
        return -__x;
    }
}

double fabs(double __x) {
    if(__x > 0) {
        return __x;
    } else {
        return -__x;
    }
}