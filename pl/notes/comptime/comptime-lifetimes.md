Zig's comptime is great, but should be type checked. Here's how to do that.
(Though it's only possible when the return value has a fixed type.)

    e ::= ... | ~e | $e | global e
    t ::= ... | ~'a t | &'a t

    Γ |- e : t   -- comptime
    Γ |-a e : t  -- runtime with lifetime 'a

    'a <: 'b
    Γ |-b e : t
    ---------------
    Γ |- ~e : ~'a t

    'a <: 'b
    Γ |- e : ~'a t
    --------------
    Γ |-b $e : t

    Γ |- e : t
    ---------------------------
    Γ |-a global e : &'static t

Lifetimes are required because of mutation! E.g.

    function leak(static mut list) {
        let x = 1;
        static list.push([x]); // type error: x does not live long enough
    }
