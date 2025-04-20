typedef struct{long a[4];long b,c,d;} PT;//semicolons in struct definition could trip up compiler

int main(){
    PT x;
    x.a[0] = 0;
    x.a[3] = 3;
    x.b = 4;
    x.c = 5;
    x.d = 6;

    if(x.a[0] != 0){
        return 1;
    }
    if(x.a[3] != 3) {
        return 2;
    }
    if (x.b != 3 || x.c != 4 || x.d != 6) {
        return 3;
    }

    return 0;
}