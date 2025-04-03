int check_stack_aligned(void) {
    //will crash on misalignment
    __asm__ __volatile__(
        // Allocate 16 bytes on stack and ensure they're 16-byte aligned
        "sub $16, %%rsp\n"
        // MOVAPS requires 16-byte alignment
        "movaps %%xmm0, (%%rsp)\n"
        // Restore stack pointer
        "add $16, %%rsp\n"
        ::: "memory"
    );
    
    // If we get here, the stack was properly aligned
    return 1;
}