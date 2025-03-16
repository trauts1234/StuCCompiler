#include <stdio.h>

enum Square {EMPTY, NOUGHT,CROSS};

enum Square OtherSide(enum Square x) {
    if (x == NOUGHT){
        return CROSS;
    }
    return NOUGHT;
}

char SquareToString(enum Square s) {
    if (s == EMPTY){
        return ' ';
    }
    if (s == NOUGHT) {
        return 'O';
    }
    if (s == CROSS) {
        return 'X';
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
            if(board[x][y] == EMPTY) {
                printf("|%d", x + 3*y + 1);
            } else {
                printf("|%c", SquareToString(board[x][y]));
            }
        }
        puts("|");
    }
    puts("------");
}

void MakeMove(enum Square board[3][3], enum Square to_place) {
    puts("enter the location of your next move");
    int location = -1;
    scanf("%d", &location);

    location = location - 1;

    int y = location / 3;
    int x = location % 3;

    if(board[x][y] != EMPTY) {
        puts("someone has already played on that square");
        MakeMove(board, to_place);
    }

    board[x][y] = to_place;
}

int IsWinner(enum Square board[3][3], enum Square side) {
    for(int i=0;i<3;++i) {
        _Bool horisontal_winner = 1, vertical_winner = 1;
        for(int j=0;j<3;++j) {
            if(board[i][j] != side) {
                vertical_winner = 0;//missing a counter when scanning vertically
            }
            if(board[j][i] != side) {
                horisontal_winner = 0;
            }
        }
        if(horisontal_winner || vertical_winner){
            return 1;
        }
    }
    
    _Bool diagonaldown = 1, diagonalup = 1;
    for(int x=0;x<3;++x) {
        if(board[x][x] != side){
            diagonaldown = 0;
        }
        if(board[x][2-x] != side) {
            diagonalup = 0;
        }
    }
    if (diagonaldown || diagonalup) {
        return 1;
    }

    return 0;
}

int main() {
    enum Square board[3][3];

    ResetBoard(board);

    enum Square side = NOUGHT;
    while(1) {
        PrintBoard(board);
        MakeMove(board, side);

        side = OtherSide(side);

        if(IsWinner(board, NOUGHT)){
            puts("nought wins!");
            PrintBoard(board);
            return 0;
        }
        if(IsWinner(board, CROSS)) {
            puts("cross wins!");
            PrintBoard(board);
            return 0;
        }
    }
    
    return 0;
}