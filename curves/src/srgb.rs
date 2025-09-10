use argparse::FromCommandLine;

/// 16-bit SRGB colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color(pub [u16; 3]);

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
            [
                4096 * parse_component(&s[0..1])?,
                4096 * parse_component(&s[1..2])?,
                4096 * parse_component(&s[2..3])?,
            ]
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
