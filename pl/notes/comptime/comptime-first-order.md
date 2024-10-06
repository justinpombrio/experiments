

    e     ::= ...
            | ~e    // runtime
            | #e    // comptime

    param ::= ...
            | x     // pass by value
            | &x    // pass by shared refn
            | *x    // pass by mutable refn
            | ~x    // pass runtime code
            | ~&x   // pass shared runtime code
            | ~*x   // pass mutable runtime code

    ~( f(expr) )
    ===
    ~(
        let x = expr;
        #f(~x)
    )

    ~( f{expr} )
    ===
    ~( #f(~expr) )
