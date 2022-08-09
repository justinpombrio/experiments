use crate::arith::Point;

const RADS_PER_TURN: f64 = 2.0 * std::f64::consts::PI;

/// A Lindenmayer system for constructing a fractal curve
#[derive(Clone, Copy)]
pub struct LindenmayerSystem {
    pub start: &'static str,
    pub rules: &'static [(char, &'static str)],
    pub len: fn(usize) -> usize,
}

struct CurveIter {
    system: LindenmayerSystem,
    depth: usize,
    at_start: bool,
    stack: Vec<&'static str>,
    point: Point<f64>,
    direction: f64,
    // hack for z-order curve
    z_index: usize,
}

impl CurveIter {
    fn new(system: LindenmayerSystem, depth: usize) -> CurveIter {
        CurveIter {
            stack: vec![system.start],
            system,
            depth,
            at_start: true,
            point: Point { x: 0.0, y: 0.0 },
            direction: 0.0,
            z_index: 0,
        }
    }
}

impl Iterator for CurveIter {
    type Item = Point<f64>;

    fn next(&mut self) -> Option<Point<f64>> {
        if self.at_start {
            self.at_start = false;
            return Some(self.point);
        }
        while let Some(top) = self.stack.last() {
            let mut chars = top.chars();
            let letter = match chars.next() {
                None => {
                    self.stack.pop();
                    continue;
                }
                Some(letter) => letter,
            };
            *self.stack.last_mut().unwrap() = chars.as_str();
            match letter {
                'l' => self.direction -= 0.25,
                'r' => self.direction += 0.25,
                'p' => self.direction -= 1.0 / 6.0,
                'q' => self.direction += 1.0 / 6.0,
                'f' => {
                    self.point.x += (self.direction * RADS_PER_TURN).cos();
                    self.point.y += (self.direction * RADS_PER_TURN).sin();
                    return Some(self.point);
                }
                'z' => {
                    self.z_index += 1;
                    let mut x = 0;
                    let mut y = 0;
                    for i in 0..16 {
                        x += (1 << i) & (self.z_index >> i);
                        y += (1 << i) & (self.z_index >> (i + 1));
                    }
                    return Some(Point {
                        x: x as f64,
                        y: y as f64,
                    });
                }
                'A'..='Z' => if self.stack.len() < self.depth + 1 {
                    self.stack.push(self.system.lookup(letter));
                }
                _ => panic!("LindermayerSystem: '{}' not recognized. (Remember: replacement letters must be capitalized.)", letter),
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.system.len(self.depth);
        (len, Some(len))
    }
}

impl ExactSizeIterator for CurveIter {}

impl LindenmayerSystem {
    /// Return the sequence of (x, y) points in the `n`th iteration of this fractal curve.
    pub fn expand(&self, depth: usize) -> impl ExactSizeIterator<Item = Point<f64>> {
        CurveIter::new(*self, depth)
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

    fn len(&self, depth: usize) -> usize {
        (self.len)(depth)
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
