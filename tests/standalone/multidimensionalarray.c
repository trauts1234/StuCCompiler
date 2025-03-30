#include <stdio.h>

enum Square {EMPTY, NOUGHT,CROSS};

void ResetBoard(enum Square board[3][3]) {
    for(int i=0;i<3;++i) {
        for(int j=0;j<3;++j) {
            board[i][j] = EMPTY;
        }
    }
}

int main() {
    enum Square board[3][3];

    ResetBoard(board);

    for(int i=0;i<3;++i) {
        for(int j=0;j<3;++j) {
            if(board[i][j] != 0){
                return 1;
            }
        }
    }
    return 0;
}