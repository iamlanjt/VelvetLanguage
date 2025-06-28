# Velvet Programming Language

**Velvet** is a programming language I am writing in order to learn Rust, and Programming Language Theory more generally. Considering this is being written before the parser is even started, Velvet is going to be an interpreted language. I will try to balance performance with ease of use, but at the end of the day, whatever helps me learn the theory of languages better is what's going to be implemented. Velvet will likely have a V2 which will be compiled, with various fixes of bad implementations (as there are bound to be).

Lexical analysis, parsing, and interpretation are all hand-written features, and no intermediary tools are used.

# The perfect theoretical Velvet example
```vel
// Mutable binding of 0 as an i32 to "my_counter"
bindm my_counter as i32 = 0
-> add(one as i32, two as i32) => i32 {
    my_counter += one + two
    ; my_counter
}

-> main() => i32 {
    bindm result as i32 = add(2, 2)
    print(result)
    ; 0
}
```