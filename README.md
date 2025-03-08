# StuCCompiler

StuCCompiler is a little project of mine that compiles a subset of C99 to NASM assembly

## Prerequisites
Before building the compiler, ensure you have the following dependencies installed:
- **Cargo** (for compiling the compiler)
- **NASM** (for assembling the generated assembly code)
- **ld** (for linking the binaries)

## Running the compiler

Build the compiler:
```sh
cargo build
```

## Usage
To compile a simple C program:

```sh
./target/debug/StuCCompiler2 main.c -o main

```
