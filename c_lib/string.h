/* Copy N bytes of SRC to DEST.  */
#include <stddef.h>
extern void *memcpy (void* __dest, void* __src,
    size_t __n);

extern int memcmp (void*__s1, void*__s2, size_t __n);

extern int strcmp (char *__s1, char *__s2);

extern size_t strlen (char *__s);