

## Ideas:

Every type is sized.

All data is tree-shaped, with a unique owner. It can be owned `T`, shared `&T`,
or mutable `*T`. Users can define type aliases that involve `&` or `*`, so that
e.g. a Rust string slice type could be user-defined (user-defined fat pointers).

Built in syntax for common monads:

- `A -> B amb &T` means `(A, &T) -> B`
- `A -> B amb *T` means `(A, *T) -> B`

Q: how do you abstract over these crazy fn types?
