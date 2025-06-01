#include <stdint.h>
#include <stddef.h>

#define empty(_T) {0} // zero-initializes object. T is required for Skye implementation, so the compiler's type checking can work properly
#define sizeOf(EXPR) sizeof(EXPR) 

typedef void* voidptr;
typedef uint8_t u8;
typedef int8_t i8;
typedef uint16_t u16;
typedef int16_t i16;
typedef uint32_t u32;
typedef int32_t i32;
typedef float f32;
typedef uint64_t u64;
typedef int64_t i64;
typedef double f64;
typedef size_t usz;

#if __WORDSIZE == 64
    #define SIZE_T_C(c)	c ## ULL
#else
    #define SIZE_T_C(c)	c ## U
#endif

#define WINDOWS 0
#define LINUX 0
#define MAC_OS 0
#define UNIX_LIKE 0

#if defined(_WIN32) || defined(__CYGWIN__)
    #undef WINDOWS
    #define WINDOWS 1
#elif defined(__linux__)
    #undef LINUX
    #define LINUX 1
    #undef UNIX_LIKE
    #define UNIX_LIKE 1
#elif defined(__APPLE__) && defined(__MACH__)
    #undef MAC_OS
    #define MAC_OS 1
    #undef UNIX_LIKE
    #define UNIX_LIKE 1
#elif defined(unix) || defined(__unix__) || defined(__unix)
    #undef UNIX_LIKE
    #define UNIX_LIKE 1
#else
    // TODO add other platforms
#endif
