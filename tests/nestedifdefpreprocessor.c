#define foo

#ifdef foo

int g() {
    return 1;
}

#ifdef bar
#error 1
#else
int f() {
    return 68;
}
#endif

#else
#error 2
#endif

int main(){
    return f() + g();
}