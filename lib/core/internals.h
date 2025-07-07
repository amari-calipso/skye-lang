#include <stdint.h>

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
