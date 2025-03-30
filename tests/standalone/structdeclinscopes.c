struct x {int a;int b;};

int main() {
    if(1){
        struct y {int c;};

        int i=0;
        while(i<5) {
            struct z {int d;};

            struct y foo;
            struct z bar;
            foo.c = 0;
            bar.d = 10;

            for(int;foo.c<bar.d;i) {
                struct a {int a;int b;};

                struct x baz;

                return 0;
            }
        }
    }
}