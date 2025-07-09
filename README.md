![](logo.png)

# Skye
Skye's programming language (Skye, for short) is the retrofuturistic systems programming language.

Note: the language is currently in a very early stage! the standard library is very limited in functionality, and the language is widely untested.

# Who is Skye?
Skye loves programming, and they enjoy writing their programs from scratch, just like you would do using C. However, they also think that while the C programming language is great, it's missing some tools and constructs to make their life easier. They do like manual memory allocation, but sometimes it's too much to handle. They like having control over all the code they write, but they would also like to have some more abstraction, as long as it doesn't hurt the runtime performance! If this sounds like you, then you have your answer: you are Skye.

# Tell me more!
Skye tries to give you a similar experience to writing code in C, but with some handy tools like type inference, generics, sum types, a more modern syntax, and a type system that's way more robust than C's, as well as a more coherent ecosystem. In some way, Skye is covering the use case for C++, but it isn't as annoying to use. At the same time, Skye is also a fairly simple language in its structure, that means that every component of it is hackable and accessible: Skye loves open source!

# Installation
To install Skye, you can either jump to the releases and download the latest version for your platform, or download the source and compile it using `cargo build --release`.

When using the Skye compiler, the `SKYE_PATH` environment variable should be set. It has to be set to the path of the compiler executable and the `lib` folder. If not set, Skye will try to infer it from the compiler executable location.

# Hello, World!
```
fn main() {
    @println("Hello, World!");
}
```

Check out more examples [here](https://github.com/amari-calipso/skye-lang-examples)

# Projects
Creating a new project in Skye is simple!

If your project is a simple one that doesn't need any specific compiler flag, you can just create a new file containing your Skye code, and then compile it or run it directly by using `skye compile <file>` or `skye run <file>` respectively.

If you're working with a bigger project (this is the most common case, since you'll be working with C compilers) you can create a Skye project by using the `skye new` command. At this stage, you should choose if you want to create a standalone program (`skye new standalone <project_name>`), or a Skye package (`skye new package <project_name>`).

Standalone projects can be built by using the `skye build` command, and Skye packages can be exported using `skye export`. The result of `skye export` is a `zip` file that can be installed using `skye install <package_file>`. To remove an installed package, use `skye remove <package_name>`.

The Skye package manager has no notion of versions, so feature-wise versioning should be performed by the developer through different package names (for example "myPackage-v1_0", "myPackage-v1_1"...). This way, projects that require a specific version of a package as a dependency don't collide with a different version of the same package while the required one is being installed.

# Comments
```
// This is a comment

/*
    This is a multiline comment
    It can't be nested
*/
```

# Variables
```
let a = 0; // Skye will infer the type for this variable
let b: u64 = 0; // You can manually specify types
const c = 3; // This variable is immutable, it cannot be modified
let d: f32; // Variables can be left undefined, but the type needs to be specified
```

# Primitive types
```
Integers:
i8 i16 i32 i64
u8 u16 u32 u64
usz (equivalent to size_t)

Floats:
f32 f64

Other:
char
voidptr (void*, mostly for C interop)
```
No implicit casting is performed, every cast must be performed explictly using the `@cast` macro.
```
let a: i32 = 0;
let b = @cast(u64, a);
```

The default integer type is `i32`, but it's possible to specify the integer type on the literal level, by putting the type after the number. For example:
```
let a = 10u64;
let b = 255u8;
let c = 1f32;
let d = 2.19083f64;
```
It's also possible to create integers in binary, octal, and hexadecimal bases.
```
let bin = 0b111010;
let oct = 0o17356;
let hex = 0x37f8A;
```
# Arrays
There are three main types of arrays in Skye: the slice, the stack array, and the heap array.

A slice is a read-only view inside another collection. You can create a slice using this syntax:
```
let mySlice: Slice[i32] = {1, 2, 3};
```

A stack array is a statically sized array allocated on the stack, or within a struct. To create one, you can use this syntax:
```
let myStackArray: [i32; 3];
let myOtherStackArray = [2 + 2; 5]; // this will create an array of 5 items, all 4s
let yetAnotherStackArray = [1, 2, 3];
```

A heap array is a dynamically sized list allocated on the heap. To create one, you can use this syntax:
```
let myArray: Array[f32] = @array(1.0, 2.0, 3.0);
```

Creating empty slices and arrays with this syntax is not permitted. To create an empty array:
```
let myEmptyArray = Array::new[f32]();
```

# Strings
There are two main types of strings in Skye: raw strings, and strings.

A String is defined by using quotes (") around your text:
```
let myString: String = "This is a string";
let stringLength = myString.length; // 16
```
The String type in Skye is not null terminated and stores its length separately. Effectively, a Skye string is just a `Slice` of `char`s.

A raw string is mostly used for C interop. It's like a C string, but not null terminated.
```
let myRawString: *const char = `This is a raw string\0`;
let rawStringLength = core::utils::cStringLength(myRawString); // 22
```

# Conditionals
Conditionals in Skye accept any numeric type as their condition, just like in C.
## If statements
```
if 2 + 2 == 4 {
	const a = true;
	if (a) @println("True!");
}
```
## While loops
```
let a = 2;
while a-- {
    @println("Looping");
}

a = 3;
do {
    @println("Looping yet again");
} while a--;
```
## For loops
There are two types of for loops in Skye.
### C-like for
```
for let i = 0; i < 10; i++ {
    @println("This will be printed 10 times");
}
```
### Foreach
```
const mySlice = {1, 2, 3};
for element; mySlice {
    @println("This will go through all the elements of the slice");
}
```
Foreach loops can iterate any type that either contains a `next` method returning an `Option`, or an `iter` method that returns a valid type containing a `next` method;

All loops can use `continue` and `break` statements.
## Switch statements
```
let a: u8 = 2;
switch a {
    3 | 4 | 5 {
        @println("Nope!");
    }
    // you can use an arrow instead of a block if you want to use a single statement for a case
    0 -> @println("Still nope");
    2 -> @println("Here!");
    else {
        @println("Something else");
    }
}
```

Using types as conditions for a switch statement allows you to compare a type against other types at compile time. An example of this is in the section about [generics](#generics);
# Functions
To create a function, you can use the `fn` keyword, like so:
```
fn add(a: i32, b: i32) i32 {
    return a + b;
}

fn sayHello() {
    @println("Hello!");
}
```

Functions can be declared, in case you need to reference one before it's actually defined
```
fn b(x: i32);

fn a(x: i32) {
    b(x - 1);
}

fn b(x: i32) {
    if x < 2 {
        a(x);
    }
}
```
You can create function bindings for existing C functions by using the `#bind` qualifier
```
#bind fn malloc(size: usz) voidptr;
```
Overloading is not allowed, however it's possible to bind different behaviors to the same function called with different types through [generics](#generics).

Variable parameter length is not allowed, however it's possible to create [macros](#macros) to call functions with a variable amount of arguments:
```
fn printAllFunction(strings: Slice[String]) {
    for string; strings {
        @println("%", string);
    }
}

macro printAll(strings*) printAllFunction(strings);
```

It's possible to create function pointers either by referencing an existing function or using the function pointer type.
```
let aFunctionPointer: fn (i32) void = a;
aFunctionPointer(3);
```
# Pointers
There are two types of pointers in Skye: the raw pointer, and the reference.

Pointers are their own type. They point to a location in memory, support pointer arithmetics, and behave as an indipendent type.
On the other hand, references internally work like pointers, but they just operate as the underlying data type. For example, if you have two references to `i32`s, you can add them directly without dereferencing them, because the compiler does it automatically.

Pointers and references also have their own `const`ness associated to them. If a pointer or a reference are `const`, the value they point to cannot be mutated.

Pointer and reference types are created with the prefix `*` and `&` operators respectively, and a `const` keyword can be added to create a `const` pointer or reference.

```
let a = 2;
// the address pointed by these pointers can be mutated
let aPtr: *i32 = &a; // a can be mutated through this pointer
let aConstPtr: *const i32 = &a; // a cannot be mutated through this pointer
let anotherConstPtr: *const i32 = &const a; // you can also use the `&const` operator to create a const reference, which can be casted to a pointer

// the address pointed by these pointers cannot be mutated
const constAPtr: *i32 = &a; // a can be mutated through this pointer
const constAConstPtr: *const i32 = &a; // a cannot be mutated through this pointer

let b = 3;
const refA: &const i32 = &a; // a cannot be mutated through this reference
const refB: &i32 = &b; // b can be mutated through this reference

let result = refA + refB; // equivalent to a + b (= 5)
```

If a function parameter is defined as a reference, the compiler will automatically create a reference for you if the function gets passed the value directly.

```
fn add(const a: &const i32, const b: &const i32) i32 {
    return a + b;
}

fn main() {
    const a = 2;
    const b = 3;

    // these are both valid
    const result = add(a, b); // here, the compiler will automatically pass the values by reference
    const resultAgain = add(&a, &b);
}
```

# Qualifiers
It's possible to use C qualifiers on function and variable declarations using the `#` operator
```
#inline
fn add(a: i32, b: i32) i32 {
    return a + b;
}

#volatile let a = 3;
```
# Defer
The `defer` statement is used to execute a statement while exiting the current scope.
```
fn test() f32 {
    let anArray = Array::new[f32]();
    defer anArray.free();

    anArray.push(1.0);
    anArray.push(2.0);

    return anArray[0];
    // anArray.free() will be called here
}
```
# Structs
```
struct MyStruct {
    myField: i32,
    const anotherField: u64
}
```

You can create a bitfield by specifying bit size for reach field

```
struct MyBitfield {
    a[2]: u8,
    b[128]: MyType
}
```

# Unions
Unions are mostly meant for C interop.
```
union MyUnion {
    a: i32,
    b: f32
}
```
# Enums
```
enum ClassicEnum {
    Variant1,
    Variant2
}

// by default, enum variants are typed `i32`,
// but you can specify a custom time using the `as` keyword
enum U64Enum as u64 {
    Variant1,
    Variant2
}

enum EnumWithCustomValues {
    Variant1 = 0,
    Variant2
}

enum SumTypeEnum {
    Variant1(i32),
    Variant2(f64)
}

enum SumTypeEnumWithCustomValues {
    Variant1(i32) = 0, // custom values will be applied to the kind enum
    Variant2(f64)
}
```
Any sum type includes a `kind` field that indicates the active variant.
```
struct Dog {}
struct Cat {}

enum Animal {
    DogVariant(Dog),
    CatVariant(Cat),
    AnotherAnimal
}

fn test() {
    let var = Animal::DogVariant(Dog.{});
    let kind = var.kind; // Animal::Kind::DogVariant;
    let dog = var.DogVariant;

    var = Animal::AnotherAnimal;
    kind = var.kind; // Animal::Kind::AnotherAnimal;
}
```

It's possible to bind all user defined types to C defined types with the following syntax:
```
struct MyStructBinding: CStructName {
    x: f32,
    y: f32
}

enum MyEnumBinding: CEnumName {
    FIRST_FIELD,
    SECOND_FIELD
}

bitfield MyBitfieldBinding: CBitfieldName {
    a: 23,
    b: 1
}

union MyUnionBinding: CUnionName {
    a: i32,
    b: f32
}
```
Structs, bitfields, and unions can be initialized through a compound literal:
```
let myStructInstance = MyStructBinding.{ x: 1.0, y: 2.0 };
let a = 2;
// field name can be omitted when it collides with the expression name
let myBitfieldInstance = MyBitfieldBinding.{ a, b: 1 };
let myUnionInstance = MyUnionBinding.{ a }; // only one field of a union can be initialized
```
# Impl
Structs and sum type enums can have methods, and they can be implemented using the `impl` keyword.
```
struct MyStruct {
    myField: i32,
    const anotherField: u64
}

impl MyStruct {
    fn new(myField: i32, anotherField: u64) Self {
        return MyStruct.{ myField, anotherField };
    }

    // self doesn't need type specifiers!
    fn add(const self) i32 {
        return self.myField + @cast(i32, self.anotherField);
    }

    fn setMyField(self, field: i32) {
        self.myField = field;
    }

    fn staticMethod() {
        @println("This method does not depend on the instance");
    }
}
```
Methods can be called either through the type with a `::` operator, or through its instances, through the `.` operator.
```
MyStruct::staticMethod();
let instance = MyStruct::new(10, 10);
let result = instance.add();
instance::setMyField(&instance, result);
```
# Namespaces
Namespaces can be created to avoid name conflicts and organize code. They can be accessed through the `::` operator and defined like this:
```
fn ambiguous() {
    @println("I am the wrong one :c");
}

namespace myNamespace {
    fn test() {
        @println("test!");
    }

    fn callTest() {
        test(); // test!
    }

    fn ambiguous() {
        @println("I am the right one!");
    }

    fn callAmbiguous() {
        // when something globally outside the namespace is called the same way
        // as something inside your namespace, Skye will pick by default what's outside.
        ambiguous(); // I am the wrong one :c

        // if you want to disambiguate, specify the entire path!
        myNamespace::ambiguous(); // I am the right one!

        // you can also use this shorthand syntax, if the thing you want to refer to
        // is defined in the same namespace
        ::ambiguous(); // I am the right one!
    }
}

// myNamespace::test();
```
# Use
The `use` statement is used to create aliases for types and identifiers.
```
use f32 | f64 as Floats;
use myNamespace::test; // in case of namespaces accesses, `as` can be omitted and the alias will be bound to the outermost name, in this case, "test"

use myNamespace::test as myTestAlias;

macro defineAdd(constant) {
    fn addValue[T: AnyFloat](x: T) T {
        return a + constant;
    }
}

// using "_" as an identifier forces the compiler to evaluate the expression without creating an alias.
use @defineAdd(1) as _; // this is especially useful for metaprogramming with macros
use addValue[f32] as _; // or, for instance, this syntax will create the necessary code for add[f32], adding to the resulting C source
```
# Import
The import statement can import both Skye packages and C libraries.
```
import "os"; // using the name with no extension will assume this is an installed package
import "otherFile.skye"; // using the full file name will search in the project folder
import "anotherFile.h";
import <"math.h">; // using angular brackets is equivalent to doing the same in C through an #include
import <<"core/internals.h">>; // using double angular brackets forces the import to address to the installed packages
```

You can also use `extern` to define libraries to link with.
```
extern pthread; // will automatically pass -lpthread
```
# Generics
Structs, sum type enums, and functions can use generics to accept multiple types
```
struct MyStruct[T] {
    a: T,
    b: T
}

impl[T] MyStruct[T] {
    fn new(a: T, b: T) Self[T] {
        return Self.{ a, b };
    }
}

// generics can have type bounds
fn add[T: AnyInt | AnyFloat](a: T, b: T) T {
    return a + b;
}

// it's possible to specify a default type for generics
enum Result[T, U = i32] {
    Ok(T),
    Error(U)
}

let myStruct = MyStruct::new(1i32, 2i32); // Skye can infer generic types...
let result = add[i32](2, 2); // ...but you can also specify types manually
```

You can use generics to give the function different behaviors depending on types.

```
fn which32[T: u32 | i32 | f32](x: T) {
    switch T {
        u32 -> @println("got a u32");
        i32 -> @println("got a i32");
        f32 -> @println("got a f32");
        else -> @unreachable;
    }
}
```

# Results and Options
Skye avoids the usage of `null` types and propagates errors by value.
```
fn someIfPositive(x: i32) ?i32 { // ?i32 corresponds to core::Option[i32]
    if x < 0 {
        return (?i32)::None;
    }

    return (?i32)::Some(x);
}

fn errorIfNegative(x: i32) u32!i32 { // u32!i32 corresponds to core::Result[u32, i32]
    if x < 0 {
        return (u32!i32)::Error(x);
    }

    return (u32!i32)::Ok(x);
}

fn main() !i32 { // omitting the left value makes the compiler assume it's `void`
   let result = try errorIfNegative(-2); // the try operator propagates the error if there is one
   // when using the try operator, error types need to match

   return (!i32)::Ok;
}
```
# Macros
It's possible to create macros in Skye, and unlike in C, they are based on the AST instead of using a preprocessor. It's also possible to bind to C macros.
```
macro constantNumber 32;
macro count(n) {
    for let i = @cast(@typeOf(n), 0); i < n; i++ {
        @println("%", i);
    }
}

macro addTwo(x) x + 2;

// C macro bindings
macro __WORDSIZE -> u8;
macro A_C_MACRO(x, y) -> i32;
```
To reference macros, the `@` operator must be used.
```
let number = @costantNumber;
@saySomething("hello!");
let result = @A_C_MACRO(1, 1);
```

To reference macros inside namespaces, this syntax is used:
```
namespace myNamespace {
    macro constantNumber 32;
}

// myNamespace::@constantNumber
```

You can create macros with variable parameter length using the following syntax:
```
macro variableArgumentsMacro(args*) {
    // `args` will be bound to a `Slice` of whatever arguments it got passed
}
```
# Interfaces
It is possible to create interfaces with types known at compile time. Interfaces allow to group shared behavior to a shared data type. Internally, this is just syntax sugar around sum types, implementing enum dispatch.
```
struct Dog {}
impl Dog {
    fn speak(const self) {
        @println("Woof!");
    }
}

struct Cat {}
impl Cat {
    fn speak(const self) {
        @println("Meow!");
    }
}

interface Animal {
    fn speak(const self);
} for Dog, Cat;

fn main() {
    let animal = @cast(Animal, Dog.{}); // you can convert an instance of a type to a compatible interface using a cast
    const dog = @cast(Dog, animal).unwrap(); // casting the interface back to its type can fail, so it may return none

    animal.speak(); // Woof!

    animal = @cast(Animal, Cat.{});
    animal.speak(); // Meow!
}
```
You can also provide a default implementation:
```
...

struct AnotherAnimal {}

interface Animal {
    fn speak(const self) {
        @println("<insert animal noise here>");
    }
} for Dog, Cat, AnotherAnimal;

fn main() {
    const animal = @cast(Animal, AnotherAnimal.{});
    animal.speak(); // <insert animal noise here>
}
```
Interfaces can be forward declared when needed, by just omitting default implementations and the `for types...` part.

This approach to type dispatching has been experimented with in Rust, and has shown up to a 10x speed increase over Rust's native dynamic dispatching, as well as much better possibility for compiler optimizations ([reference](https://gitlab.com/antonok/enum_dispatch)).

# Main operators
| name | syntax | additional notes |
| ---- | ------ | ----------- |
| Prefix increment | `++x` | Increments `x` before it's used [*1](#additional-information) |
| Suffix increment | `x++` | Increments `x` after it's used [*1](#additional-information) |
| Prefix decrement | `--x` | Decrements `x` before it's used [*1](#additional-information) |
| Suffix decrement | `x--` | Decrements `x` after it's used [*1](#additional-information) |
| Negation | `-x` | ... |
| Boolean not | `!x` | Can also define a `Result` type with `Ok = void` |
| Bitwise not | `~x` | ... |
| Reference | `&x` | Returns a reference to `x`. Can also define a reference type if applied to a type |
| Const reference | `&const x` | Returns a const reference to `x` (`x` cannot be modified through that reference). Can also define a const reference type if applied to a type |
| Dereference | `*x` | Dereferences a pointer. Can also define a pointer type if applied to a type [*4](#additional-information) |
| Const dereference | `*const x` | Dereferences a pointer and returns a `const` value. Can also define a const pointer type if applied to a type [*4](#additional-information) |
| Option | `?x` | Defines an `Option[x]` type where `x` is a type |
| Try | `try x` | Returns the `Ok` or `Some` value of `x` where `x` is a `Result` or `Option`. Propagates the `Error` or `None` if the set variant is not `Ok` or `Some` |
| Access | `x.y` | Accesses the `y` property of `x`, whether it's a method or a field, where `x` is an instance of a struct, sum type enum, union or bitfield. Automatically dereferences pointers if necessary [*4](#additional-information) |
| Static access | `x::y` | Accesses the `y` property of `x` statically, where `x` is a namespace, a struct type, an enum type, or an instance of the above. This operator will automatically follow pointers at compile time if necessary |
| Addition | `x + y` `x += y` | ... |
| Subtraction | `x - y` `x -= y` | ... |
| Multiplication | `x * y` `x *= y` | ... |
| Division | `x / y` `x /= y` | [*3](#additional-information) |
| Modulo | `x % y` `x %= y` | [*3](#additional-information) |
| Shift left | `x << y` `x <<= y` | Shifts `x` left `y` times |
| Shift right | `x >> y` `x >>= y` | Shifts `x` right `y` times |
| Boolean or | <code>x &#124;&#124; y</code> | ... |
| Boolean and | `x && y` | ... |
| Bitwise xor | `x ^ y` `x ^= y` | ... |
| Bitwise and | `x & y` `x &= y` | ... |
| Bitwise or | <code>x &#124; y</code> <code>x &#124;= y</code> | Can define a type group if the operands are types |
| Greater | `x > y` | ... |
| Greater or equal | `x >= y` | ... |
| Less | `x < y` | ... |
| Less or equal | `x <= y` | ... |
| Equality | `x == y` | ... |
| Inequality | `x != y` | ... |
| Result | `x ! y` | Defines a `Result[x, y]` type where `x` and `y` are types |
| Ternary | `x ? y : z` | Returns `y` if `x` is truthy, otherwise returns `z` |

## Operator overloading
It's possible to perform operator overloading by creating some special functions in your types.
```
struct Vector {
    x: f32,
    y: f32
}

impl Vector {
    fn __add__(const self, const other: &const Self) Self {
        return Vector.{ x: self.x + other.x, y: self.y + other.y };
    }
}
```

Here is a list of operators that can be overloaded

| operator | method | n. of arguments (except self) | return type |
| -------- | ------ | ----------------------------- | ----------- |
| `++{}` or `{}++` | `__inc__` | 0 | void |
| `--{}` or `{}--` | `__dec__` | 0 | void |
| `+{}` | `__pos__` | 0 | any |
| `-{}` | `__neg__` | 0 | any |
| `!{}` | `__not__` | 0 | any |
| `~{}` | `__inv__` | 0 | any |
| `*{}` | `__deref__` or `__constderef__` [*2](#additional-information) | 0 | pointer to any |
| `*const {}` | `__constderef__` or `__deref__` [*2](#additional-information) | 0 | pointer to any |
| `{} + {}` | `__add__` | 1 | any |
| `{} - {}` | `__sub__` | 1 | any |
| `{} / {}` | `__div__` | 1 | any |
| `{} * {}` | `__mul__` | 1 | any |
| `{} % {}` | `__mod__` | 1 | any |
| `{} << {}` | `__shl__` | 1 | any |
| `{} >> {}` | `__shr__` | 1 | any |
| <code>{} &#124;&#124; {}</code> | `__or__` | 1 | any |
| `{} && {}` | `__and__` | 1 | any |
| `{} ^ {}` | `__xor__` | 1 | any |
| <code>{} &#124; {}</code> | `__bitor__` | 1 | any |
| `{} & {}` | `__bitand__` | 1 | any |
| `{} > {}` | `__gt__` | 1 | any |
| `{} >= {}` | `__ge__` | 1 | any |
| `{} < {}` | `__lt__` | 1 | any |
| `{} <= {}` | `__le__` | 1 | any |
| `{} == {}` | `__eq__` | 1 | any |
| `{} != {}` | `__ne__` | 1 | any |
| `{} += {}` | `__setadd__` | 1 | any |
| `{} -= {}` | `__setsub__` | 1 | any |
| `{} /= {}` | `__setdiv__` | 1 | any |
| `{} *= {}` | `__setmul__` | 1 | any |
| `{} %= {}` | `__setmod__` | 1 | any |
| `{} <<= {}` | `__setshl__` | 1 | any |
| `{} >>= {}` | `__setshr__` | 1 | any |
| `{} ^= {}` | `__setxor__` | 1 | any |
| <code>{} &#124;= {}</code> | `__setor__` | 1 | any |
| `{} &= {}` | `__setand__` | 1 | any |
| `{}[{}]` | `__subscript__` or `__constsubscript__` [*5](#additional-information) | any | pointer to any |

Additionally, Skye offers you copy constructors and destructors, mostly used for special types like smart pointers. They are respectively the `__copy__` method and the `__destruct__` method. The Skye compiler will warn you when it inserts calls to those methods inside the code, so that eventual debugging is easier.

### Additional information
1) Prefix and suffix increments and decrements are handled by the Skye compiler, and thus prevent undefined behavior for cases where multiple increments are used in the same expression or statement. Every expression is evaluated from left to right, and the outcome is always predictable.
2) `__constderef__` is used when dereferencing from a const source, which in some cases throws an error if using `__deref__`.
3) In debug mode (the default compilation mode), division and modulo operators do not cause undefined behavior, but rather they panic the program if division by zero is performed. This check is disabled in release mode for performance reasons.
4) Unlike in C, in debug mode, dereferencing a `null` pointer, either explicitly or implictly, will result in a panic rather than undefined behavior. This check is disabled in release mode for performance reasons.
5) `__constsubscript__` is used when subscripting from a const source, which in some cases throws an error if using `__subscript__`.
