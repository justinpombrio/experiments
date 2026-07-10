### Tuples

_A tuple is just nested pairs._

For example, a triple `a, b, c` is exactly the same as two nested pairs
`(a, (b, c))`. Tuples are written without parens, so this is just how parsing
works. This avoids problems like the fact that Rust's `Eq` trait is defined only
on tuples up to length 12
[link](https://doc.rust-lang.org/std/primitive.tuple.html).

### Function Pattern Bindings

_You can bind variables in function parameters._

You should be able to write:
```
fn transpose(array: [[#T: Type * #M: Usize] * #N: Usize]): [[T * N] * M])
```

instead of 
```
fn transpose(#T: Type, #M: Usize, #N: Usize,
  array: [[T: Type * M] * N]): [[T * N] * M]
): [[T * N] * M]
```

The semantics are that you bind the expected type pattern to the actual type,
recursively bottom-up left-to-right, adding bindings as you go. A type variable
must be bound before it's used.

### No Booleans. Only Option.

_Standard boolean ops instead act on options._

- `true` and `false` are shorthand for `Ok ()` and `Err ()`.
- `if A then B` evaluates `B` is `A` is Ok.
- `A else B` evaluates `B` is `A` is Err.
- `A and B` is Ok if both `A` and `B` are.
- `A or B` is Ok if either `A` or `B` is.
- `not A` swaps Ok and Err.

https://justinpombrio.net/2025/09/22/imagining-a-language-without-booleans.html
