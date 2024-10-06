## Patterns, `and`, `or`, `let`, etc.

`and` and `or` should be keywords, for a few different reasons:

- They're very short English words with _exactly_ the indented meaning. That's
  so rare! We should take advantage of it!
- They short circuit, so they can't be written as a function. They're the only
  operators in Rust that can't be overloaded.
- They interact with other syntax. E.g. it would be nice to have
  `if let Some(x) = x and x > 0`.

```
while let Some(x) = iter.next() { ... }

if let Some(x) = x_opt and x > 0 { ... }
// real-life example:
if let KeyCode::Char(ch) = self.code and self.modifiers.shift { ... }

let Some(x) = opt_x else { return None; }
```

## Strings

String literals can have some tricky requirements:

- Including arbitrary characters.
- Having multi-line string literals, without the indentation resetting to zero.
- Doing either of the above while maintaining the property that if one character
  is about another in the string, it's above the other on the screen too.

Zig has a beautiful solution to this. There are two kinds of string literals:

- regular strings surrounded by double quotes that include escape sequences,
- multi-line string literals that begin with `"""` (Zig uses `\\` but `"""` is
  obviously better).

```
// String literals
let normal_string = "hello!";
let complex_string =
    """// string literals
    """let normal_string = "hello!";
    """let complex_string =
    """    """ let normal_string = "hello!";
    """    """ ... etc ...
    """    ;
    ;
```

## Comments

Zig does well with comments too: `//` for regular comments and `///` for doc
comments, which must appear before a comment-able element. No multi-line
comments, so that lines can be tokenized independently.
