int f(int x){
    return x;
}

int main(){
    for(int i=f(1);i<10;i = i + f(1)) {
        return 0;
    }
    return 1;
}