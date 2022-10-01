use crate::arith::{Bounds, Point};
use std::collections::HashMap;

const RADS_PER_TURN: f64 = 2.0 * std::f64::consts::PI;

/// A Lindenmayer system for constructing a fractal curve.
///
/// The curve of iteration N is defined by:
///
/// - Start with the string `start`.
/// - Do N times: Replace each capital letter in the string with its replacement listed in `rules`.
/// - Follow the instructions described by the lowercase letters and +/- now in the string.
///
/// The meanings of the lowercase letters are:
///
///     - -- turn left by `self.angle`
///     + -- turn left by `self.angle`
///     f -- move forward by 1.0
///     z -- magically jump to the next point in the z-order curve
///
/// If `implicit_f` is true, treat all capital letters in the _final_ string as if they were `f`.
#[derive(Clone, Copy)]
pub struct LindenmayerSystem {
    pub start: &'static str,
    pub rules: &'static [(char, &'static str)],
    pub angle: f64,
    pub implicit_f: bool,
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
                '-' => self.direction -= self.system.angle/360.0,
                '+' => self.direction += self.system.angle/360.0,
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
                } else if self.system.implicit_f {
                    self.point.x += (self.direction * RADS_PER_TURN).cos();
                    self.point.y += (self.direction * RADS_PER_TURN).sin();
                    return Some(self.point);
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

    /// Determine the bounds of this curve. Walks the whole curve!
    pub fn bounds(&self, depth: usize) -> Bounds<f64> {
        let mut points = self.expand(depth);
        let first_point = points.next().unwrap();
        let mut bounds = Bounds {
            min: first_point,
            max: first_point,
        };
        for point in points {
            bounds.min.x = bounds.min.x.min(point.x);
            bounds.min.y = bounds.min.y.min(point.y);
            bounds.max.x = bounds.max.x.max(point.x);
            bounds.max.y = bounds.max.y.max(point.y);
        }
        bounds
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
        // Initialize a letter-count map from every letter to 0
        let mut letter_counts = HashMap::new();
        letter_counts.insert('f', 0);
        letter_counts.insert('z', 0);
        for letter in self
            .start
            .chars()
            .chain(self.rules.iter().flat_map(|(_, s)| s.chars()))
        {
            letter_counts.entry(letter).or_insert(0);
        }

        // Count the letters in `self.start`
        for letter in self.start.chars() {
            *letter_counts.get_mut(&letter).unwrap() += 1;
        }

        // Update the letter counts based on the rewrite rules, for each iteration of the curve
        for _ in 0..depth {
            let mut new_letter_counts = letter_counts.clone();
            for (seek, replace) in self.rules {
                *new_letter_counts.get_mut(seek).unwrap() -= letter_counts[seek];
                for letter in replace.chars() {
                    *new_letter_counts.get_mut(&letter).unwrap() += letter_counts[seek];
                }
            }
            letter_counts = new_letter_counts;
        }

        // The length of the curve is the number of forward steps we take, plus 1
        let mut length = letter_counts[&'f'] + letter_counts[&'z'] + 1;
        if self.implicit_f {
            for (letter, count) in letter_counts {
                if letter.is_uppercase() {
                    length += count;
                }
            }
        }
        length
    }
}

#[test]
fn test_curves() {
    use crate::CURVES;

    let hilbert_curve: LindenmayerSystem = CURVES[0].1;

    assert_eq!(
        hilbert_curve
            .expand(2)
            .map(|point| (point.x.round() as i32, point.y.round() as i32))
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
