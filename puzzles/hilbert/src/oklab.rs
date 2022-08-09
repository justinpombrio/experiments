use crate::arith::interpolate;

pub type Color = [u16; 3];

/// Construct a color scale in OKLAB color space (a mapping from `[0, 1]` to `Color`).
///
/// `max_saturation`, `min_lightness`, and `max_lightness` determine the shape of a truncated cone
/// in OKLAB space:
///
/// - The cone points "down", with its tip on pure black and its circular top facing pure white.
/// - `min_lightness` is the height (L-component) at which the tip of the cone is truncated.
/// - `max_lightness` is the height of the circular top of the cone.
/// - `max_saturation` is the radius of the circular top of the cone.
///
/// `hsv` converts the `[0, 1]` input into a `(hue, sat, val)` triple. Each of these components
/// lies between 0.0 and 1.0. Together, these three components determine the location on the
/// truncated cone:
///
/// - `val` determines the height of the point, where 0 is `min_lightness` and 1 is
///   `max_lightness`.
/// - `hue` determines the horizontal angle of the point, where 0 is yellow, 0.1 is orangeish, and
/// 1 wraps around to yellow again.
/// - `sat` is the horizontal distance from the center of the cone, where 0 is centered (pure gray)
/// and 1 is on the edge of the cone.
pub struct ColorScale {
    pub max_saturation: f64,
    pub min_lightness: f64,
    pub max_lightness: f64,
    pub hsv: fn(f64) -> (f64, f64, f64),
}

impl ColorScale {
    pub fn sample(&self, f: f64) -> Color {
        const RADS_PER_TURN: f64 = 2.0 * std::f64::consts::PI;

        let (hue, sat, val) = (self.hsv)(f);
        assert!(0.0 <= hue && hue <= 1.0);
        assert!(0.0 <= sat && sat <= 1.0);
        assert!(0.0 <= val && val <= 1.0);

        let l = interpolate(val, self.min_lightness, self.max_lightness);
        let rad = sat * self.max_saturation * l / self.max_lightness;
        let a = rad * (RADS_PER_TURN * hue).sin();
        let b = rad * (RADS_PER_TURN * hue).cos();
        let (color, is_clamped) = oklab_to_srgb([l, a, b]);
        if is_clamped {
            panic!("Color is out of bounds. Try reducing saturation.");
        }
        color
    }
}

/// Convert from the OKLAB color space to srgb. Returns an srgb color,
/// and a boolean indicating whether the color was clamped (whether it
/// was out of bounds).
pub fn oklab_to_srgb(lab: [f64; 3]) -> (Color, bool) {
    let l = lab[0] + 0.3963377774 * lab[1] + 0.2158037573 * lab[2];
    let m = lab[0] - 0.1055613458 * lab[1] - 0.0638541728 * lab[2];
    let s = lab[0] - 0.0894841775 * lab[1] - 1.2914855480 * lab[2];

    let l = l * l * l;
    let m = m * m * m;
    let s = s * s * s;

    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let mut b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

    // Hacky adjustment to increase color uniformity.
    b *= 0.9;

    let (r, clamped_r) = to_gamma(r);
    let (g, clamped_g) = to_gamma(g);
    let (b, clamped_b) = to_gamma(b);

    ([r, g, b], clamped_r || clamped_g || clamped_b)
}

fn to_gamma(u: f64) -> (u16, bool) {
    let g = if u >= 0.0031308 {
        1.005 * u.powf(1.0 / 2.4) - 0.055
    } else {
        12.92 * u
    };
    let component = (g * (u16::MAX as f64)).round();
    clamp(component)
}

fn clamp(component: f64) -> (u16, bool) {
    if component < 0.0 {
        (0, true)
    } else if component > (u16::MAX as f64) {
        (u16::MAX, true)
    } else {
        (component as u16, false)
    }
}

#[test]
fn test_oklab() {
    /*
    // L / max valid (a,b)-radius
    // 0	0.034
    // 0.05	0.02
    // 0.1	0.02
    // 0.15	0.027
    // 0.2	0.036
    // 0.25	0.044
    // 0.3	0.053
    // 0.35	0.062
    // 0.4	0.07
    // 0.45	0.079
    // 0.5	0.088
    // 0.55	0.097
    // 0.6	0.106
    // 0.65	0.114
    // 0.7	0.123
    // 0.75	0.132
    // 0.8	0.104
    // 0.85	0.077
    // 0.9	0.05
    // 0.95	0.025
    // 1	0
    fn radius(l: f64) -> f64 {
        const PI: f64 = std::f64::consts::PI;

        for r in (0..1000).map(|n| (n as f64) / 1000.0) {
            for angle in (0..360).map(|n| 2.0 * PI * (n as f64) / 360.0) {
                let a = r * angle.cos();
                let b = r * angle.sin();
                let (_, clamped) = oklab_to_srgb([l, a, b]);
                if clamped {
                    return ((r * 1000.0) - 1.0) / 1000.0;
                }
            }
        }
        unreachable!();
    }
    for l in (0..21).map(|n| (n as f64) / 20.0) {
        println!("{}\t{}", l, radius(l));
    }
    assert!(false);
    */

    assert_eq!(oklab_to_srgb([0.0, 0.0, 0.0]), ([0, 0, 0], false));
    assert_eq!(
        oklab_to_srgb([1.0, 0.0, 0.0]),
        ([62258, 62258, 62258], false)
    );
    assert_eq!(
        oklab_to_srgb([0.5, 0.0, 0.0]),
        ([24087, 24087, 24087], false)
    );
    assert_eq!(
        oklab_to_srgb([0.75, 0.13, 0.0]),
        ([58604, 33427, 41565], false)
    );
    assert_eq!(
        oklab_to_srgb([0.75, 0.0, -0.13]),
        ([34584, 41201, 62594], false)
    );
}
