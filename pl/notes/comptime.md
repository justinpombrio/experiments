## Comptime

**Challenge:** When are two types equal?

**Answer:** When they have the same expression.
    No hidden mutation -> (same expr -> same type).

**Challenge:** How can the compiler avoid re-computing the same type-related
info multiple times? E.g. info associated with the type; specialization.

**Answer:** `struct` caches its output based on its inputs. They're guaranteed
to be simple bulitin types (Type, Usize), so hashing them is easy. (Maybe also
cache based on the srcloc.

Although... we also want arbitrary comptime associated data, which will need a
user-supplied hash function.

**Challenge:** Isn't type equality really hard?
What about `Array(T, n + m)` vs. `Array(T, m + n)`?

**Answer:**
```
let x: Array(T, n + m) = ...;
let y: Array(T, m + n) = cast x as Array(T, m + n);
```
`cast` checks _at comptime_ that the two types are the same. (Not merely that
they have the same layout, that they're literally the same type.)

### Inlining vs. Monomorphizing

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

7 * dist_sq_2(y)
```

Monomorphization requires you to be able to hash the comptime args so that you can memoize them.

You can mix monomorphization and inlining, but that's kind of weird, so there should be just those
two choices. Inlining should be the default, because (i) it doesn't require the args to implement
Hash, and (ii) if all args are comptime, then inlining is the same as a regular function call.

### Conclusions

- A function with comptime args is inlined by default.
- If you mark a function as `const/pure/mono/cached/memo`, it's monomorphized instead. One copy is
  made for each distinct set of comptime args, cached in a HashMap.
