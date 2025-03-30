int main() {
    unsigned char a = -1;
    //TODO short
    unsigned int b = -1;
    unsigned long long c = -1;

    if(a != 255) {
        return 1;
    }

    if (b != 4294967295) {
        return 2;
    }

    if (c != 18446744073709551615ULL) {
        return 3;
    }

    return 0;
}