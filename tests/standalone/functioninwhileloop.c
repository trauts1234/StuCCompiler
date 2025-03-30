int f(int x){
    return x;
}

int main(){
    int i=0;
    
    while(i<f(1)) {
        return 0;
    }
    return 1;
}