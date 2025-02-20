#define foo

#ifdef foo

int g() {
    return 1;
}

#ifdef bar
#error
#else
int f() {
    return 68;
}
#endif

#else
#error
#endif

int main(){
    return f() + g();
}