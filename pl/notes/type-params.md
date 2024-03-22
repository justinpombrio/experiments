## Type Parameters

There are no type parameters! Instead, args can be used in type position:

    fn new_array(T: $Type, N: $usize, Default: fn() -> T) -> Array(T, N)

and then passed in like normal comptime args:

    new_array(String, X + 1, String.Default): Array(String, X + 1)

The type checking rules are:

- Any value used in type position must be comptime ($).
- If a type variable is used more than once (e.g. `T` above), the type checker
  checks that all of the _expressions_ passed into that position are
  _equivalent_.

When are two expressions equivalent? It's not syntactic equality! For example,
`n + 1 === 1 + n` but `f() !== f()`. Function calls are never equivalent, unless
the function is marked as `cached` (or `pure`?) and its args are equivalent. The
compiler will memoize calls to `cached` functions:

    // No knowledge of what type comes out, but if you call this twice with the
    // same input you know the _same_ type will come out.
    cached fn transform_type(T: Type) -> Type

QUESTION: What is the `Array` function used above?

    cached fn Array(T: $Type, N: $usize) -> ??? { ??? }
