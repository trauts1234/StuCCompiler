int a = 0;

int b = 1 * 2;

int c = 1 + 2;

int d = 1 - 2;

int e = 10/2 + 1;

int main() {
    if (a != 0) {
        return 1;
    }

    if (b != 2) {
        return 1;
    }

    if (c != 3) {
        return 1;
    }

    if (d != -1) {
        return 1;
    }

    if (e != 6) {
        return 1;
    }

    return 0;
}