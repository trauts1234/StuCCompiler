int f(){
    enum x {A,B};
    return A;
}

int g() {
    enum x {X,Y,Z};
    return Z;
}

int main(){
    if (f() != 0) {
        return 1;
    }
    if(g() != 2) {
        return 1;
    }

    return 0;
}