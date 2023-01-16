# Examples

∀ - compile-time function; arg can be used in type position
∃ - compile-time existential; 'impl' in Rust not 'dyn'; no vtable
λ - run-time function; not a closure, no captures
δ - run-time existential; 'dyn' in Rust; vtable

## Collection of Entities

    type EntityCollection = Map
        TypeId
        (δ T:Type, _:Display T. Vec T)

## Print

    println("Hello {} of age {}seconds", name, age);

What's the type of `println`? Perhaps:

    println<"Hello {} of age {}seconds">([name, age]);

    println<S: String, infer N: Int>([args: ∃T.T; N]) {
        static {
            Vec<FormatParam> Params = format_params(S);
            if params.len() != N { panic("wrong number of args!"); }
            for i in 0..N {
                unseal args[i] as ∃T. arg {
                    if T != params[i] { panic("bad arg!"); }
                    if let Some(D: Display T) = IMPL_TABLE[T]["Display"] {
                        dyn write<D>(args[i]);
                    } else {
                        panic("can't display arg!");
                    }
                }
            }
        }
    }

Or:

    println<"Hello {} of age {}seconds">([name, age]);

## 
