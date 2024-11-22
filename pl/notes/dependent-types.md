## Dependent Types

Dependent types are when the _type_ of one thing depends on the _value_ of another. E.g.:

- `fn f(#T: Type, v: T) -> (T, T)`
- `{ #T: Type, #Show: fn(T)->String, val: T }`
- `fn make-array(#N: usize) -> [i32; N]`

Since you can now use variables in types, it is natural that you can put arbitrary expressions in
types:

```
fn append(
    #T: Type, #N: usize, #M: usize,
    x: [T; N], y: [T; M]
) -> [T; N + M]
```

The thing that's hard about dependent types is type equality. How can you tell if two _expressions_
are equal? CoQ says they're equal when they evaluate to the same thing. Nuprl lets you prove
equalities yourself. Also, is `f(3)` equal to `f(3)`? If `f` isn't pure, it might not be!

I propose a really simple method for determining type equality.

Type expressions are only equal if they're syntactically equal _and_ all functions they contain are
`const`. This is extremely conservative; it doesn't even tell you that `1 + 2` is equal to `2 + 1`.
So there's a way to get around this dynamically: you use `view..as` to declare that one type
expression is equal to another. Whether they really are equal is checked at comptime:

```
fn add_appends(
    #T: Type, #N: usize, #M: usize,
    x: [T; N], y: [T; M]
) -> [T; N + M] {
    let xy: [T; N + M] = append(x, y);
    let yx: [T; M + N] = append(y, x);
    let yx_view = #view yx as [T; N + M];
    map_array(xy, yx_view, fn(x, y) { x + y })
}
```

### Conclusions

- Types are abitrary expressions, and may contain comptime variables.
- Two type expressions are equal iff (i) they're syntactically identical, and (ii) all of the
  functions that they contain are `const` (monomorphized).
- `#view e as t` asserts at comptime that typeof(e) == t, and has value `e` and type `t`.
  (Specifically: record typeof(e) during type checking, then evaluate both typeof(e) and t at
  comptime and assert that they're equal.)
