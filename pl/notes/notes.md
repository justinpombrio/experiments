## Ideas

Every type is sized.

Not subtyping!

All data is tree-shaped, with a unique owner. It can be owned `T`, shared `&T`,
or mutable `*T`. Users can define type aliases that involve `&` or `*`, so that
e.g. a Rust string slice type could be user-defined (user-defined fat pointers).

Built in syntax for common monads:

- `A -> B amb &T` means `(A, &T) -> B`
- `A -> B amb *T` means `(A, *T) -> B`

Q: how do you abstract over these crazy fn types?

## Controversial ideas

All types are owned, like in Rust.

Functions can borrow their arguments, but cannot _return_ borrowed data.
I thought of this independently of Val, but they have a good writeup:
https://www.val-lang.dev/

## Four quantifiers

∀ - compile-time function; arg can be used in type position
∃ - compile-time existential; 'impl' in Rust not 'dyn'; no vtable
λ - run-time function; not a closure, no captures
δ - run-time existential; 'dyn' in Rust; vtable
