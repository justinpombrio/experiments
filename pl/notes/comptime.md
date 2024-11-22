## Comptime

There are two things you can do with functions with comptime args:

- Inlining
- Monomorphization

For example, start with a partially comptime function:

```
fn dist_sq(#x: i32, y: i32) -> i32 {
    #let x2 = #x * #x
    let y2 = y * y
    #x2 + y2
}
```

Inlining looks like this:

```
7 * dist_sq(2, y)
==>
7 * {
    let y2 = y * y
    4 + y2
}
```

Monomorphization looks like this:

```
7 * dist_sq(2, y)
==>
fn dist_sq_2(y: i32) -> i32 {
    let y2 = y * y
    4 + y2
}
...

2 * dist_sq_2(y)
```

Monomorphization requires you to be able to hash the comptime args so that you can memoize them.

You can mix monomorphization and inlining, but that's kind of weird, so there should be just those
two choices. Inlining should be the default, because (i) it doesn't require the args to implement
Hash, and (ii) if all args are comptime, then inlining is the same as a regular function call.

### Conclusions

- A function with comptime args is inlined by default.
- If you mark a function as `const/pure/mono/cached/memo`, it's monomorphized instead. One copy is
  made for each distinct set of comptime args, cached in a HashMap.
