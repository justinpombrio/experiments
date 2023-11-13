An interface is a struct passed at comptime, and an implementation of an
interface is a value of that type. For example:

    fn Add[T: Type] = struct Add {
        add: T, T -> T
    }

    fn double[infer T: Type, A: Add[T] = T.Add](x: T) -> T {
        A.add(x, x)
    }

See:

    https://lobste.rs/s/kvgb5f/all_you_need_is_data_functions#c_wl7vt7
