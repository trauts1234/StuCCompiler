int main() {
    char x = 0;

    x = 'A';

    if(x != 65) {
        return 1;
    }

    x = '\n';

    if(x != 10) {
        return 2;
    }

    x = '\\';

    if(x != 92) {
        return 3;
    }

    return 0;
}