int f(char x[6]){
    return x[0];
}

int main(){
    if(f("hello") != 104) {
        return 1;
    }
    char* x = "hello";

    if(f(x) != 104) {
        return 2;
    }

    return 0;
}