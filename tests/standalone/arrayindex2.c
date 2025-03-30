int main() {
    long long int x[3];

    x[0] = 0;
    x[1] = 1;
    x[2] = 2;

    for(int i=0;i<3;++i) {
        if(x[i] != i){
            return i+1;
        }
    }
    return 0;
}