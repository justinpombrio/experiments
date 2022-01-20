## MetaLanguage

    v ::= var

    b ::= val               -- ints and such
        | [bs]              -- seq
    bs ::= b ... b

    t ::= b                 -- bytecode
        | Expr              -- { type: Type, val: Bytecode }
        | Type              -- 
        | Interface
        | { v: t, ... }
        | (t, t)            -- etc.
        | t -> t
        | forall v. e end   -- cached

    e ::= v
        | e e               -- application
        | e; e              -- seq
        | let v: t = e in e
        | { v: t = e, ... } -- module
        

Builtins:

    core.var: 
    core.add: Expr -> Expr -> !Expr
    assert_impl: Type -> Interface -> !()


## Basics

    use std.u8;
    =>
    let u8 = std.u8 in

    let x = 0;
    =>
    let x: Expr = core.int("0") in
    where
        core.int("0")
        ->
        Expr { type = std.u32, val = BYTECODE[0], }

    x => x


## Structs

    struct Person { name: Str, age: Int }
    =>
    let ty: Type = freshtype {
        name: "Person",
        layout: `(Str, Int)`
    };
    let Person = {
        $type = ty;
        $constructor = lambda(record: Record) -> Expr {
            // 1. Check that record has `name` and `age`, or error
            // 2. Check that record does not have other fields
            // 3. Construct code that turns `name` and `age` into `(name, age)`
        };
        $matcher = `forall T. |value: (Str, Int), body: {name: Str, age: Int} -> T| -> T
                             { body {name, age} }`,
        name: `|person: (Str, Int)| person.0`,
        age: `|person: (Str, Int)| person.1`
    };

    person.name
    ->
    typeof(`person`).name

    let person = Person { name: n, age: 27 };
    ->
    core.let("person", Person.$constructor({name: n, age: 27}))

Error messages on all of these functions?

## Existentials

    ∃T: I. T      -> (sizeof T, *impl I for T, Box<T>)

    ∃T: I. Vec<T> -> (sizeof T, *impl I for T, Vec<T>)

(In the first case, the size of `T` is unknown so it can't be stored directly.
But in the second case, the size of `Vec<T>` is fixed regardless of `T`, so it
can be stored?)




