## Data Types

Data definitions are given as potentially recursive product and sum types:

```
type Rgb = (r: U8, g: U8, b: U8);
type Color = ('Red: () | 'Rgb: Rgb)
```

At compile time, these are translated into product, sum, and mu types:

```
type R = (s: S, t: U8);
type S = (r: R, t: U8);
-->
type R = mu R of (s: (r: R, t: U8), t: U8);
type S = mu S of (r: (s: S, t: U8), t: U8);
```

If the mu parameters are named simply by their depth, this gives a canonical
nameless representation for every type, making type equality comparisons easy.

_Side note:_ the straightforward expansion algorithm will treat these two types
differently:
```
type X = (Y, Y); type Y = (X, X);
type Z = (Z, Z);
```
This is fine, honestly.

### Private Aliases

Instead of nominal types.

[FILL]

```
pub type Stack<T> = pub Vec<T>; // type alias
pub type Stack<T> = Vec<T>; // type alias here, abstract type elsewhere
```
