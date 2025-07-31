# Velvet Programming Language
Velvet is a flexible, easy-to-understand, open-source programming language built on an actively developed hand-written recursive descent parser, Rust powered interpreter, and developing core language ideas.

# Usage
Once you `cargo build` Velvet (or use `cargo run` directly), you can run the executable and specify a file name to evaluate.

```bat
cargo build --release
./target/release/velvet.exe my_file.vel
```

```bat
cargo run -- my_file.vel
```

# A minimal example
```
bind grocery_list as any = [
    "Apple",
    "Orange Juice",
    "Banana",
    "Orange",
    "Bagel"
]

-> is_fruit(item) => bool {
    assert_type#(item, "string")
    ; match item {
        "Apple" => true,
        "Banana" => true,
        "Orange" => true
    } ! false
}

for grocery_list_item of grocery_list do {
    print(grocery_list_item, "is a", match grocery_list_item {
        is_fruit => "Fruit",
        "Orange Juice" => "Tasty Drink"
    } ! "Mystery!!!")
}
```

Assuming everything is working correctly, you can expect your program to output the following:

```
Apple is a Fruit
Orange Juice is a Tasty Drink
Banana is a Fruit
Orange is a Fruit
Bagel is a Mystery!!!
```

# Contributing
Velvet is open-source and open to contributions by anyone! Any addition to the repo is greatly appreciated. Visit `CONTRIBUTING.md` for a guide on making your first Pull Request!

# Community
In Velvet's Discord server, there is much more information and guides, as well as a server bot which will allow you to execute Velvet straight from the server!

Join us below:

https://discord.gg/ayEz2xag2s

# Compiler Flags / Options
Due to the beta status of the compiler, some features which are considered enabled by default on most compilers might be disabled by default in Velvet.

- `cmp_do_coerce` ~ Should the compiler attempt to coerce basic values using a cast?