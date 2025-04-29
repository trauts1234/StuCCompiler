
extern void change_x();

extern int x;

int main() {
    if(x != 0) {
        return 1;
    }

    change_x();

    if(x != 1) {
        return 2;
    }

    x++;

    if(x != 2) {
        return 3;
    }

    return 0;
}