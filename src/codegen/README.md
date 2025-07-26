# Velvet LLVM IR Generation
Although Velvet comes with an interpreter (see `src/runtime`), an LLVM IR generator structure is included for educational purposes and can be switched from the main file via arguments.

To use compilation methods instead of interpretation, append `compile` as a flag.

# Incorret behavior
At the time of writing, the compiler is not fully finished and is being comitted in a beta state. I'm aware that a vast number of Velvet features will result in an unimplemented error, and that a lot of currently working features may be unstable. Use at your own free will.

# Entry point return behavior
All top-level statements are considering to be in the entry point function, and the return code of the program will be the last returned value of the scope.