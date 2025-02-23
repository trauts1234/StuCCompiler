#define x
#undef y

#if defined x || defined y
int main() {
    return 69;
}
#endif