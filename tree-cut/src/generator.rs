use rand::{rngs::StdRng, Rng, SeedableRng};

// From https://github.com/justinpombrio/partial-pretty-printer/blob/master/tests/standard/generative_testing.rs

/// An interface for types that can be randomly (or deterministically) generated, for use as test
/// inputs. Implementors must obey these requirements:
///
/// - `generate` must be a pure function. That is, it must behave deterministically (given the
///   output of `gen`), and it cannot have side effects.
/// - For any given `size`, `generate` must produce only finitely many possible values.
pub trait Generator: Copy {
    type Value;

    fn generate<P: Picker>(&self, size: u32, picker: &mut P) -> Self::Value;
}

/// The sole source of entropy in a Generator.
pub trait Picker {
    fn pick_int(&mut self, max: u32) -> u32;
}

/*********************
 * Random Generation *
 *********************/

/// Construct an infinite stream of random values of the given size.
pub fn generate_random<G: Generator>(
    generator: G,
    size: u32,
    seed: [u8; 32],
) -> impl Iterator<Item = G::Value> {
    GenRandom::new(generator, size, seed)
}

struct GenRandom<G: Generator> {
    size: u32,
    rng: StdRng,
    generator: G,
}

impl<G: Generator> GenRandom<G> {
    fn new(generator: G, size: u32, seed: [u8; 32]) -> GenRandom<G> {
        GenRandom {
            size,
            rng: StdRng::from_seed(seed),
            generator,
        }
    }
}

impl<G: Generator> Iterator for GenRandom<G> {
    type Item = G::Value;

    fn next(&mut self) -> Option<Self::Item> {
        struct PickRandom<'a>(&'a mut StdRng);

        impl<'a> Picker for PickRandom<'a> {
            fn pick_int(&mut self, max: u32) -> u32 {
                self.0.gen_range(0..max)
            }
        }

        let mut picker = PickRandom(&mut self.rng);
        Some(self.generator.generate(self.size, &mut picker))
    }
}

/****************************
 * Deterministic Generation *
 ****************************/

/// Construct a finite stream of all values of type `G::Value` of up to the given size.
pub fn generate_all_up_to_size<G: Generator>(
    generator: G,
    max_size: u32,
) -> impl Iterator<Item = G::Value> {
    (1..max_size).flat_map(move |size| generate_all_of_size(generator, size))
}

/// Construct a finite stream of all values of type `G::Value` of the given size.
pub fn generate_all_of_size<G: Generator>(
    generator: G,
    size: u32,
) -> impl Iterator<Item = G::Value> {
    GenAll::new(generator, size)
}

struct GenAll<G: Generator> {
    size: u32,
    index: usize,
    stack: Vec<(u32, u32)>,
    done: bool,
    generator: G,
}

impl<G: Generator> GenAll<G> {
    fn new(generator: G, size: u32) -> GenAll<G> {
        GenAll {
            size,
            index: 0,
            stack: vec![],
            done: false,
            generator,
        }
    }

    fn advance(&mut self) {
        self.index = 0;
        while let Some((n, max)) = self.stack.pop() {
            if n + 1 < max {
                self.stack.push((n + 1, max));
                return;
            }
        }
        if self.stack.is_empty() {
            self.done = true;
        }
    }
}

impl<G: Generator> Iterator for GenAll<G> {
    type Item = G::Value;

    fn next(&mut self) -> Option<Self::Item> {
        struct PickNext<'a> {
            stack: &'a mut Vec<(u32, u32)>,
            index: &'a mut usize,
        }

        impl<'a> Picker for PickNext<'a> {
            fn pick_int(&mut self, max: u32) -> u32 {
                if let Some((n, _)) = self.stack.get(*self.index) {
                    *self.index += 1;
                    *n
                } else {
                    assert_eq!(*self.index, self.stack.len());
                    self.stack.push((0, max));
                    *self.index += 1;
                    0
                }
            }
        }

        if self.done {
            return None;
        }

        let mut picker = PickNext {
            stack: &mut self.stack,
            index: &mut self.index,
        };
        let value = self.generator.generate(self.size, &mut picker);
        self.advance();
        Some(value)
    }
}
