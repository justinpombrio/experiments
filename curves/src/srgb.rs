use argparse::FromCommandLine;
use std::ops::{Add, Mul};

/// 16-bit SRGB colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub [u16; 3]);

impl Color {
    // Construct a color that can't be parsed, to use as a sentinel value.
    // (Yeah this is ugly.)
    pub fn sentinel() -> Color {
        Color([0, 0, 1])
    }
}

// It would be nicer to implement FromStr, but FromStr::Err doesn't require fmt::Display, so
// argparse is unable to show the error message, so it gives an unhelpful message like 'Bad value
// 7'. I therefore implement FromCommandLine to get useful error messages.
// https://doc.rust-lang.org/stable/std/str/trait.FromStr.html
impl FromCommandLine for Color {
    fn from_argument(s: &str) -> Result<Color, String> {
        fn parse_component(component: &str) -> Result<u16, String> {
            u16::from_str_radix(component, 16).map_err(|_| "color -- invalid hex digit".to_owned())
        }

        if s.len() != 6 && s.len() != 3 {
            return Err("color -- wrong length (expected 3 or 6 hex digits)".to_owned());
        }
        let components = if s.len() == 3 {
            let r = parse_component(&s[0..1])?;
            let g = parse_component(&s[1..2])?;
            let b = parse_component(&s[2..3])?;
            [256 * 17 * r, 256 * 17 * g, 256 * 17 * b]
        } else {
            [
                256 * parse_component(&s[0..2])?,
                256 * parse_component(&s[2..4])?,
                256 * parse_component(&s[4..6])?,
            ]
        };
        Ok(Color(components))
    }
}

impl Mul<f64> for Color {
    type Output = Color;

    fn mul(self, scalar: f64) -> Color {
        let [r, g, b] = self.0;
        Color([
            (scalar * r as f64) as u16,
            (scalar * g as f64) as u16,
            (scalar * b as f64) as u16,
        ])
    }
}

impl Add<Color> for Color {
    type Output = Color;

    fn add(self, other: Color) -> Color {
        Color([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
        ])
    }
}
