#define SCHAR_MAX __SCHAR_MAX__
#define SHRT_MAX __SHRT_MAX__
#define INT_MAX __INT_MAX__
#define LONG_MAX __LONG_MAX__

#define SCHAR_MIN (-__SCHAR_MAX__-1)
#define SHRT_MIN (-__SHRT_MAX__ -1)
#define INT_MIN (-__INT_MAX__ -1)
#define LONG_MIN (-__LONG_MAX__ -1L)

#define UCHAR_MAX (__SCHAR_MAX__*2 +1)
#define USHRT_MAX (__SHRT_MAX__ * 2 + 1)

#define UINT_MAX (__INT_MAX__ *2U +1U)
#define ULONG_MAX (__LONG_MAX__ *2UL+1UL)

#ifndef MB_LEN_MAX
#define MB_LEN_MAX 1
#endif

#define CHAR_BIT __CHAR_BIT__

#define LLONG_MAX __LONG_LONG_MAX__
#define LLONG_MIN (-__LONG_LONG_MAX__-1LL)
#define ULLONG_MAX (__LONG_LONG_MAX__*2ULL+1ULL)

