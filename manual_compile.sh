#!/bin/bash

nasm -felf64 -O0 -g -o a.o a.asm

ld -o a.out /usr/lib/x86_64-linux-gnu/crt1.o /usr/lib/x86_64-linux-gnu/crti.o a.o -lc /usr/lib/x86_64-linux-gnu/crtn.o --dynamic-linker /lib64/ld-linux-x86-64.so.2