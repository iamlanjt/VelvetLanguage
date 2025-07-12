# Intermediate Velvet Extension
This extension to the main Velvet package is not an essential part of execution, but can be useful in different circumstances. It aims to replace `serde` and `serde_json` in the dependencies by moving from JSON AST serialization to a custom protocol, for sake of performance improvements and package size reduction.

# What does intermediate generation consist of?
When the main executable is ran with the flag `COMPILE_INTERMEDIATE` (`velvet my_file.vel COMPILE_INTERMEDIATE`), an `out.imvel` file will be produced. You can then pass the intermediate form (as you would a normal Velvet program) to the main executable: `velvet out.imvel`

This file serves as a pre-compiled and serialized AST using binary, which can (in some use cases) allow for more performant execution.

In a compiled language, all of these size and performance results are reaped from the executable output, but considering Velvet is not compiled, this is a middle-ground. An intermediate form is the smallest file you can compile without generating an executable.

# When to use an intermediate form for execution
Use intermediate execution if...
- Package size is a primary concern over source file readability
- You are loading large / complex programs consistently, or in short succession (i.e, module loaders)

# Pros of intermediate generation
- Less overhead when loading complex and long programs which would generate a large AST
- Protocol is Velvet-specific; deserializer doesn't have to parse human-readable plaintext such as JSON
- Velvet size is reduced due to lack of bulky JSON overhead

# Cons of intermediate generation
- Program pipeline may become slower if conditions are not correct (i.e, if you are executing the intermediate form directly after generation, if your programs are not complex enough to benefit from size reduction, etc.)
- Human readability & modification is lost without the source attached
- Some Velvet flags become unusable, mainly regarding token & AST dumping