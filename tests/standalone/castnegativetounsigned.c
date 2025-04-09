int main() {
    unsigned char a = -1;
    unsigned short b = -1;
    unsigned int c = -1;
    unsigned long long d = -1;

    if(a != 255) {
        return 1;
    }

    if(b != 65535){
        return 2;
    }

    if (c != 4294967295) {
        return 3;
    }

    if (d != 18446744073709551615ULL) {
        return 4;
    }

    return 0;
}