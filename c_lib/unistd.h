/*
note that the long and unsigned long params here are 64 bit via LP64 (this is not for Windows)
*/
extern long write (int __fd, void *__buf, unsigned long __n);

extern int close (int __fd);

extern long read (int __fd, void *__buf, unsigned long __nbytes);