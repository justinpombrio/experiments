pub const HILBERT_CURVE: LindenmayerSystem = LindenmayerSystem {
    start: "A",
    rules: &[('A', "rBflAfAlfBr"), ('B', "lAfrBfBrfAl")],
};

/// A Lindenmayer system for constructing a fractal curve
pub struct LindenmayerSystem {
    pub start: &'static str,
    pub rules: &'static [(char, &'static str)],
}

impl LindenmayerSystem {
    /// Return the sequence of (x, y) points in the `n`th iteration of this fractal curve.
    pub fn expand(&self, n: usize) -> impl Iterator<Item = (f64, f64)> {
        let mut curve = self.start.to_owned();
        for _ in 0..n {
            curve = self.iter(curve);
        }
        curve = curve
            .chars()
            .rev()
            .filter(|ch| !ch.is_uppercase())
            .collect();
        CurveIter {
            at_start: true,
            point: (0.0, 0.0),
            direction: 0.0,
            curve,
        }
    }

    fn iter(&self, curve: String) -> String {
        let mut new_curve = String::new();
        for letter in curve.chars() {
            match letter {
                'l' | 'r' | 'p' | 'q' | 'f' => new_curve.push(letter),
                'A'..='Z' => new_curve.push_str(self.lookup(letter)),
                _ => panic!("LindermayerSystem: '{}' not recognized. (Remember: replacement letters must be capitalized.)", letter),
            }
        }
        new_curve
    }

    fn lookup(&self, letter: char) -> &'static str {
        for (seek, replace) in self.rules {
            if *seek == letter {
                return replace;
            }
        }
        panic!(
            "LindenmayerSystem: replacement letter '{}' not found.",
            letter
        );
    }
}

// Follow the action characters in `curve`, which is stored in reverse.
struct CurveIter {
    at_start: bool,
    point: (f64, f64),
    direction: f64,
    curve: String,
}

const RADS_PER_TURN: f64 = 2.0 * std::f64::consts::PI;

impl Iterator for CurveIter {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<(f64, f64)> {
        if self.at_start {
            self.at_start = false;
            return Some(self.point);
        }
        while let Some(action) = self.curve.pop() {
            match action {
                'l' => self.direction -= 0.25,
                'r' => self.direction += 0.25,
                'p' => self.direction -= 1.0 / 6.0,
                'q' => self.direction += 1.0 / 6.0,
                'f' => {
                    self.point.0 += (self.direction * RADS_PER_TURN).cos();
                    self.point.1 += (self.direction * RADS_PER_TURN).sin();
                    return Some(self.point);
                }
                _ => panic!("LindermayerSystem: action '{}' unrecognized.", action),
            }
        }
        None
    }
}

#[test]
fn test_curves() {
    assert_eq!(
        HILBERT_CURVE
            .expand(2)
            .map(|(x, y)| (x.round() as i32, y.round() as i32))
            .collect::<Vec<_>>(),
        vec![
            (0, 0),
            (1, 0),
            (1, 1),
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 3),
            (1, 2),
            (2, 2),
            (2, 3),
            (3, 3),
            (3, 2),
            (3, 1),
            (2, 1),
            (2, 0),
            (3, 0)
        ]
    );
}
