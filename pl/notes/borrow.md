## References

Use Mutable Value Semantics (MVS). It gives references without GC or borrow checking.

The semantics are simple. There are three types of bindings:

    let owned: String = ...;
    let *mutable: String = ...;
    let &shared: String = ...;

    fn foo(owned: String, *mutable: String, &shared: String)

The compiler just needs to check that mutable and owned values are disjoint.

The _downside_ of MVS is that you can't return or store references. One place where this is
particularly painful is not having shared or mutable iterators. Instead, you need to use internal
iteration:

    collection.each_mut(|item| *item += 1);

Likewise, if you want a method to return a reference to a `name: String` function, it needs to take
a callback instead:

    fn with_name<R>(&self: Person, callback: Fn(&str) => R) => R {
        callback(&self.name)
    }

    person.with_name(|name| {
        print("Hello {}", name)
    })

    fn with_opt_name(&self: Person, callback: *Fn(&str) => ()) => () {
        match &self.name {
            None => None,
            Some(&name) => Some(callback(&name))
        }
    }

    if person.has_name() {
        person.with_opt_name(|name| {
            print("Hello {}", name)
        })
    } else {
        print("Hello, anonymous")
    }

This is a sufficiently common pattern that it warrents a language feature to make it easier:
borrowing functions (Hylo's "subscripts").

## Borrowing Functions

A _borrowing function_ is a function that "yields" a value instead of returning it; a `yield`
statement invokes a callback[...]

    // equivalent to with_name above
    borrowing fn name[&self: Person] => &str {
        yield &self.name
    }

    borrow &name = person.name() {
        print("Hello {}", name);
    }

    // Can't do anything with opt_name
