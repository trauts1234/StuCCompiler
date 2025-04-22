#include <FILE.h>

/* Standard streams.  */
extern FILE *stdin;		/* Standard input stream.  */
extern FILE *stdout;		/* Standard output stream.  */
extern FILE *stderr;		/* Standard error output stream.  */
/* C89/C99 say they're macros.  Make them happy.  */
#define stdin stdin
#define stdout stdout
#define stderr stderr

extern int puts (char *__s);

extern int printf (char *__format, ...);
extern int scanf (char *__format, ...);

extern int dprintf (int __fd, char * __fmt, ...);
extern int fprintf (FILE * __stream, const char * __format, ...);