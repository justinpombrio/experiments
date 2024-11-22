// This is a hypothetical language; not Rust.
// But the syntax matches enough to get some highlighting.

/*******************/
/* Impossibilities */
/*******************/

// Can't just have a new type `Code(T)`, because
// comptime and runtime can be interspersed:
fn foo(list: Vec(u32), Pow: @u32) -> u32 {
    let total = 0;
    for x in list {
        for N in 1..Pow {
            x *= N;
        }
        total += x;
    }
}

let foo = |Pow: u32| {
    ~|list: Vec(u32)| -> u32 {
        let total = 0;
        for x in list {
            @for N in 1..Pow {
                ~x *= N;
            }
            total += x;
        }
        total
    }
};


/*********/
/* Types */
/*********/

i32  // runtime int (32 bits)
@i32 // comptime int (0 bits)
(@i32, i32)  // mixed pair (32 bits)
@(@i32, i32) // equivalent to (@i32, @i32) (0 bits)

// Regular runtime hashmap
HashMap(String, String)

// Technically allowed, though you'd never want it. This is a "runtime"
// hashmap, but both the keys and values are known at comptime. So
// you need the runtime hashmap value to do lookups, but every lookup
// produces the same value.
HashMap(@String, @String)

// This is a _comptime_ hashmap, because the '@' is applied
// to the whole struct, not merely its type params.
@HashMap(String, String)

// A hashmap whose keys and values are pointers into the .text segment.
HashMap(&@String, &@String)

// Closure with runtime arg
i32 -> i32
// Closure with comptime arg -- TODO: does this make sense?
(N: @usize) -> Array(N)
// A function is just a comptime closure:
//     A => B = @(A -> B)
// Function with runtime args (e.g. add)
(i32, i32) => i32
// Function with comptime arg
@i32 => @i32
// Comptime args can be used in the return type (lightweight dependent types)
(i32, N: @usize) => Array(i32, N) // the type of Array.repeat
// You can't do arithmatic in the type, even at comptime:
// (i32, N: @usize) => Array(i32, N + 1) -- TYPE ERROR
// Runtime args _cannot_ be used in the return type
// (i32, N: usize) => Array(i32, N) -- TYPE ERROR

// Type aliases.
type Point2d = (f32, f32)
type Point2d = { x: f32, y: f32 }
// Type params are implemented using ordinary type aliases.
type AssocArray = T1: @Type => T2: @Type => Vec((T1, T2))
// An interface is also implemented with a type alias.
type Display = Self: @Type => {
    fmt: Self -> String,
}
// (Self is not a keyword; it could be "T".)
// Notice how '@' changes an interface:
Display(i32) = { fmt: i32 -> String }
@Display(i32) = { fmt: i32 => String }

// Comptime existential types.
// Written using tuple syntax, though they're not really tuples.
// Rust's `impl Display` would be:
(T: @Type, @Display(T), T)
// These existential types are more general than Rust's dyn;
// you can have a "vector of some type (known at comptime) that implements Display":
(T: @Type, @Display(T), Vec(T))
// An alias for shorthand
type Impl = Trait: (@Type => @Type) => (T: @Type, @Trait(T), T)
Impl(Display)
Impl((Clone, Display))

// Runtime existential types.
// Rust's `dyn Display` would be:
(T: Type, Display(T), T)
// A vector of some type that implements Display:
(T: Type, Display(T), Vec(T))
// (`Display(T)` acts as the vtable, though it's just a regular struct.)
// An alias for shorthand
type Dyn = Trait: (@Type => @Type) => (T: @Type, Trait(T), T)
Dyn(Display)
Dyn((Clone, Display))


/********************/
/* Type Definitions */
/********************/

// In Rust: `enum ColorChannel { Red, Green, Blue }`
type ColorChannel = data is Red () or Green () or Blue ()

// `LinkedList(D)` is the type,
// `Self` is a type variable for the recursive type (_not_ a keyword),
// `Node` is the type constructor
type LinkedList = D: &Type -> data Self is Node {
    data: D,
    next: Option(Self),
}
// usage looks like this:
let list = LinkedList(i32).Node {
    data: 0,
    next: Some(LinkedList(i32).Node {
        data: 1,
        next: None,
    })
}

type Tree = Data: &Type -> data Self
    is Branch {
        data: Data,
        left: Self,
        right: Self
    }
    or Leaf {
        data: Data,
    }


/**********************/
/* Emulating Closures */
/**********************/

type Closure = (Arg: @Type, Ret: @Type)
    => (Self: @Type, Self, (Self, Arg) => Ret)

fn closure(
    infer Captures: @Type,
    infer Arg: @Type,
    infer Ret: @Type,
    captures: Captures,
    func: (Captures, Arg) -> Ret
) => Closure(Arg, Ret) {
    (Captures, captures, func)
}

fn apply(
    infer Arg: @Type,
    infer Ret: @Type,
    closure: Closure(Arg, Ret),
    arg: Arg
) -> Ret {
    let (_, captures, func) = closure;
    func(captures, arg)
}

fn make_linear_transform(a: f64, b: f64) -> Closure(f64, f64) {
    closure((a, b), fn((a, b), x: f64) -> f64 { a * x + b })
}

fn linear_transformation_example() {
    let transform = make_linear_transform(2.0, 3.0);
    apply(transform, 0.0); // 3.0
    apply(transform, 1.0); // 5.0
}


/********************/
/* Emulating Traits */
/********************/

// There are no traits, only comptime structs!

type Display = Self: @Type -> {
    fmt: (Formatter, Self) -> String,
}

fn show(
    infer T: @Type,
    value: T,
    Display: @Display(T) = T.Display, // get from `with` clause
) {
    value.fmt() // equivalent to `Display.fmt(value)`
}

type HashMap = (
    Key: @Type with {
        Hash: @Hash(Key),
        Key: @Eq(Key),
    },
    Val: @Type
    KeyHash: @Hash(Key) = ???,
    KeyEq: @Eq(Key) = ???,
) => struct Self {
    table: Vec((Key, Val)),
} with {
    fn new() -> Self {
        Self { table = Vec((Key, Val)).new() }
    }
    // shorthand for:
    // let set: fn(Self, Key, Val) -> () = fn(self, key, val) { ... }
    fn set(self: Self, key: Key, val: Val) -> () {
        // `.push` is a comptime function "`with`" Vec(_).
        self.table.push((key, val));
    }
    fn get(self: Self, key: Key) -> Option(Val) {
        ...
    }
    let Display: @Display(Self) = { ... }
}

fn HashMap(
    Key: @???,
    Val: @Type
) -> struct Self {
    table: Vec((Key, Val)),
    fn new() -> Self {
        Self {
            table: Vec((Key, Val)).new()
        }
    }
    // shorthand for:
    // let set: fn(Self, Key, Val) -> () = fn(self, key, val) { ... }
    fn set(self: Self, key: Key, val: Val) -> () {
        // `.push` is a comptime function on Vec(_).
        self.table.push((key, val));
    }
    fn get(self: Self, key: Key) -> Option(Val) {
        ...
    }
    Display: Display(Self) = {
    }
}

// Existential version
type HashMapInterface = (
    Key: @Type,
    KeyHash: @Hash(Key),
    KeyEq: @Eq(Key)
    Val: @Type
) -> (Self: @Type, {
    new: () -> Self,
    set: (Self, Key, Val) -> ()
    get: (Self, Key) -> Option(Val),
})
fn HashMap(
    Key: @Type,
    KeyHash: @Hash(Key),
    KeyEq: @Eq(Key)
    Val: @Type
) -> HashMapInterface {
    type Self = Vec((Key, Val))
    (Self, {
        fn new() -> Self {
            Self::new()
        }
        fn set(self, key, val) {
            self.push((key, val))
        }
    })
})

// Mu version
type HashMapInterface = (
    Key: struct Key { Hash: @Hash(Key), Eq: @Eq(Key) },
    Val: @Type,
) -> struct Self {
    new: () -> Self,
    get: (Self, Key) -> Option(Val),
    set: (Self, Key, Val) -> (),
    Display: @Display(Self)
})
fn HashMap(
    Key: struct Key { Hash: @Hash(Key), Eq: @Eq(Key) },
    Val: @Type
) -> HashMapInterface(Key, Val) {
    struct Self {
        ???
    }
}
// A shorthand version of this:
fn HashMap(
    Key: struct Key { Hash: @Hash(Key), Eq: @Eq(Key) },
    Val: @Type
) struct Self {
    ???
}

fn example_custom_hash() {
    let HM = HashMapInterface((String.0, String.1 with { Hash = MyStringHash }));
    ...
}

type VectorInterface = T: @Type -> (Self: @Type, {
    new: () -> Self,
    push: (Self, T) -> (),
    Display: @Display(Self),
})

fn Vector(T: @Type) -> VectorInterface(T) {
    type Self = {
        len: usize,
        elems: Array(T),
    }
    (Self, {
        new = |()| {
            len = 0,
            elems = Array(T).new()
        },
        push = |(self, elem)| {
            if self.len + 1 >= self.elems.len() {
                self.elems.resize((self.elems.len() * 2).max(1))
            }
            self.elems[self.len] = elem;
        },
        Display = {
            fmt = |f, self| {
                writeln!(f, "[");
                writeln!(f, "{}", self.elems);
                writeln!(f, "]");
            }
        }
    })
}

type Pair = T: @Type -> U: @Type -> (T, U)

// What does `Pair.Display =` mean??
Pair.Display = fn(
    T: @Type,
    U: @Type,
    DisplayT: @Display(T) = T.Display,
    DisplayU: @Display(U) = U.Display,
) -> Display(Pair(T, U)) {
    {
        fn fmt(self: Pair(T, U)) -> String {
            let (t, u) = self;
            "(" + DisplayT.fmt(t) + ", " + DisplayU.fmt(u) + ")"
        }
    }
}


/*********************/
/* Emulating Modules */
/*********************/

// There are no modules, only comptime structs!





// Basic types, runtime and comptime($):

// Some runtime values
let num: i32 = 0
let text: String = "hi"
let record = { num: i32 = 0, text: String = "hi" }

// Some comptime values (like `const` in Rust)
let num_comptime: $i32 = 0
let text_comptime: $String = "hi"
let record_comptime = { num: $i32 = 0, text: $String = "hi" }

// You can mix runtime & comptime in the same "value"
let record_mixed = { num: i32 = 0, text: $String = "hi" }
// ^-- at runtime, takes up only the space of an i32

// Function types. For simplicity, there's no closures.
let inc: (n: u32) -> u32 = |n| n + 1
let array_repeat: (v: i32, N: $u32) $-> Array(i32, N) = |v, N| {
    let array = $Array(i32, N).zeros();
    $for i in 0..N {
        array[i] = v;
    }
}
// With an invocation of `array_repeat(x, 3)`, the for loop would expand into:
array[0] = x;
array[1] = x;
array[2] = x;
// If you used `for` instead of `$for`, the loop would not be unrolled.



let shapes = Vec((S: $Type, Vec(S), Shape(S)))

Let Shapes = Vec((S: Type, `Vec(S)`, Shape(S))).new([




type Square = {
    side_len: f32,
}

type Circle = {
    radius: f32,
}

type Shape = $S: Type $-> {
    name: $String,
    area: S $-> f32,
}

let $SquareShape: Shape(Square) = {
    name = "square",
    area = |sq: Square| $-> f32 {
        sq.side_len * sq.side_len
    }
}

let $AreaShape: Shape(Circle) = ...

let squares = Vec(Square).new([
    Square { side_len: 1.0 },
    Square { side_len: 2.0 },
])
let circles = ...

let shapes = $Vec(($S: Type, Vec(S), Shape(S))).new([
    (Square, squares, SquareShape),
    (Circle, circles, CircleShape),
])

for (_, shapes, Sh) in shapes {

}





type Square = {
    side_len: f32,
}

type Circle = {
    radius: f32,
}

type Shape(S: Type) = {
    area: S -> f32,
}

Let SquareShape: Shape(Square) = {
    area = |sq: Square| -> f32 {
        sq.side_len * sq.side_len
    }
}

Let CircleShape: Shape(Circle) = ...

let squares = Vec(Square).new([
    Square { side_len: 1.0 },
    Square { side_len: 2.0 },
])
let circles = Vec(Circle).new(...)

Let Shapes = Vec((S: Type, `Vec(S)`, Shape(S))).new([
    (Square, squares, SquareShape),
    (Circle, circles, CircleShape),
])


