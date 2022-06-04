

# TODO foo
    fn id<A>(a: A) -> A;
    -> mod id<A> { fn self(a: A) -> A }

    struct Pair<A, B>(A, B);
    -> mod Pair<A, B> { type self = struct (A, B); }

## Hash Example

    sig Type where
        type Self;
    end

    sig Hash<T: Type> where
        fn hash(val: T) -> u32;
    end

    mod StringHash : Hash<String> where
        fn hash(val: String) -> u32 { ... }
    end
    String.Hash = StringHash;

    mod HashMap<K: Type, V: Type, H: Hash<K>> where
        type Self = Vec<(K, V)>;
        fn new() -> Self { ... }
        fn insert(self: *Self, key: K, val: V) { ... }
    end
