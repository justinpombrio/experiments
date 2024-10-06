fn add(x: ~i32, y: ~i32) -> ~i32 {
    ~(x + y)
}

fn add_const(x: i32, y: ~i32) -> ~i32 {
    ~(#x + y)
}

fn HashMap(
    Key: Type where fn Hash(Self) -> i32,
    Val: Type
) -> infer {
    struct HashMapImpl(Vec<(Key, Val)>)
    where {
        fn new() -> ~Self {
            ~HashMapImpl(Vec::new())
        }

        fn insert(*~self, key: ~Key, val: ~Val) {
            ~self.0.push((key, val))
        }

        fn lookup(
            &~self,
            key: &~Key, 
            callback: ~fn(&Val) -> R,
            infer R: Type,
        ) -> R {
            ~if let Some(i) = self.0.index(key) {
                callback(&self.0[i].1)
            }
        }
    }
}
