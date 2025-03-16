#include <stdio.h>

enum Square {EMPTY, NOUGHT,CROSS};

char SquareToString(enum Square s) {
    if (s == EMPTY){
        return '*';
    }
}

void ResetBoard(enum Square board[3][3]) {
    for(int i=0;i<3;++i) {
        for(int j=0;j<3;++j) {
            board[i][j] = EMPTY;
        }
    }
}

void PrintBoard(enum Square board[3][3]) {
    puts("------");
    for(int y=0;y<3;++y) {
        for(int x=0;x<3;++x) {
            printf("|%c", SquareToString(board[x][y]));
        }
        puts("|");
    }
    puts("------");
}

int main() {
    enum Square board[3][3];

    ResetBoard(board);

    
    return 0;
}