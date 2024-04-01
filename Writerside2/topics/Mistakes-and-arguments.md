# Resolving mistakes and arguments

Copilot will make mistakes when generating rust code.

The good thing is that the compiler and the tests written, expose these mistakes quite quickly, often the code will not compile.
Sometimes the code will compile, but may deadlock.

Copilot can reason about deadlocks, if you tell it, this function appears to hang, when called like 
this it will recommend ways to rewrite the code.

Rust is very strict about ownership of variables, code can fail, because of errors with the way parameters are handed to functions.

You can get into a back and forth argument between the compiler and copilot.

Copilot will `reason` about the compilers error messages, and provide a convincing explanation, then the new code
will fail, often copilot will switch between two concepts for the code, and almost be trapped in a loop.

Some of the code that copilot tries to write, it cannot write correctly.
Copilot is literally an expert at rust, it does not hesitate to use complex functions, so understanding where the error lies in some complex code can be difficult for the referee.

I find as the `referee` in the argument, I need to tell copilot sometimes to break expressions down into simpler steps
to isolate where the error is introduced.



