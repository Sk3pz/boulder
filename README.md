# boulder

Boulder is designed to be a multi-purpose systems programming language that resembles rust and kotlin, and will be primarily designed for bare-metal applications to be run directly on hardware, and for embedded software. Boulder is designed to have no object oriented features, instead staying similar to C with just functions, structs, and enums. Structs will be able to have accessor methods by using the 'self' keyword in impl statements like rust, which will be basically the same as having a function take a reference to a struct in c. The standard library will be written in a way to allow for use on complete baremetal, and will include basic support for general hardware. Each part of the standard library will be able to be turned off through compile arguments. Everything down to the heap allocator will be togglable for the most extreme usecases. Need to write software on such limited hardware that you are considering writing assembly yourself? boulder is the perfect fit for you.

Boulder uses the `.rock` extension for files

### Implemented:
✅ Input Stream\
✅ Lexer\
✅ Parser\
✅ Import Statements (other files)\
❔ Compiling to C (Currently Working On)\
❌ Compiling to ASM\
❌ Compiling to Machine Code\
❌ Sediment (Package manager for Boulder libraries and programs)

Syntax example of Boulder (currently using rust syntax because of the similarity for syntax highlighting): 
```rust
use "print.rock"

macro BINARY_NUMBER_FAIL = 0x04

fn start() {
    println("This will print out on any supported OS!")
    // call a system interrupt (This isnt meant do do anything specific, just an example of syntax and features.)
    @0x00
    if true {
        @0x01
    } else {
        @0x02
    }

    // binary numbers
    let binary_number = 0b010010101100
    let mask = 0b000000001111
    let result = binary_number & mask

    if result != 0b000000001100 {
        // ? is for panic. if the program reaches this, it will dump all used memory and stop.
        // The message will:
        //  if logging is enabled, log to a file
        //  if printing is possible, print to the screen,
        //  Attempt to return it.
        // the panic function will take anything as a parameter.
        ? BINARY_NUMBER_FAIL
    }

}
```
