## With

### With solves

- Methods:
    ```
    x: T with fn f(T, U) -> V
    y: U
    -------------------------
    x.f(y): V
    ```

- Hiding:
    ```
    let T' = T without U
    ```
- Overriding:
    ```
    int with fn __lt__(x, y) { x > y }
    ```
- Implicit methods:
    ```
    fn f(infer #Elem: Type with ...)
    ```

### Overlapping Interfaces in Rust

The maintainer of Rust's `iter-tools` has an issue, that Rust keeps breaking the library. Here's
what happens.

Rust's `Iterator` type lacks a convenient method. For example, there's no way to interleave two
iterators (like A1, B1, A2, B2, A3, B3, ...). `iter-tools` fixes this by implementing methods like
`.interleave()` on its own trait called `IterTools`. It declares that all types that implement
`Iterator` also implement `IterTools`. Then users can `use iter_tools::IterTools` and get all the
methods for free.

This is all well and good until the Rust standard library decides that `.interleave()` is a nice
method and adds it to `Iterator`. This is considered a backwards compatible change because it's
simply adding a method to an existing trait. But it causes all code that uses `iter-tools`'
`.interleave()` method to break, because now it's ambiguous whether it refers to
`iter_tools::IterTools::interleave()` or to `Iterator::interleave()`.

This problem arises because there's no ordering between `Iterator` and `IterTools`. But maybe there
can be, by making interfaces lexically scoped like most other things in a language?

### The With Feature

    struct Add(Self: #type) {
        Result: type;
        Zero: Self;
        fn add(self: Self, other: Self) -> Self;
    }

    let rec Int = BUILTIN_INT with Add(Int) {
        Result: Int,
        Zero: 0,
        fn add(self: Int, other: Int) -> Int {
            BUILTIN_INT_ADD(self, other)
        }
    };

    type ::= ... | type with { S, ... }
        where S has the form `struct _(Self: #type) { ... }`

    T with U with V  ===  T with (U union V)
    T with S_1 with S_2  ===  T with S2  // this solves the itertools issue
