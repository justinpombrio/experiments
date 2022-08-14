/// 16-bit SRGB colors.
pub type Color = [u16; 3];

/// Convert from HSV OkLab to SRGB.
///
/// The input is HSV components that describe a point in the OkLab color space using polar
/// coordinates:
///
/// - `hue` determines the horizontal angle of the point, where 0 is yellow, 0.1 is orangeish, and
///   1 wraps around to yellow again.
/// - `sat` is how colorful the point is: 0 is pure gray, and higher numbers are more colorful.
///   To be precise, `sat * val` is the distance from the white/black line.
/// - `val` determines the height of the point, where 0 is black and 1 is white.
///
/// The output is 16-bit `[r, g, b]` components in the SRGB color space.
pub fn oklab_hsv_to_srgb([hue, sat, val]: [f64; 3]) -> Option<Color> {
    const RADS_PER_TURN: f64 = 2.0 * std::f64::consts::PI;
    let l = val;
    let rad = sat * val;
    let a = rad * (RADS_PER_TURN * hue).sin();
    let b = rad * (RADS_PER_TURN * hue).cos();
    oklab_to_srgb([l, a, b])
}

/// Convert from the OKLAB color space to srgb. Returns an srgb color,
/// or None if the color is out of bounds.
pub fn oklab_to_srgb(lab: [f64; 3]) -> Option<Color> {
    let (color, clamped) = try_oklab_to_srgb(lab);
    if clamped {
        None
    } else {
        Some(color)
    }
}

fn try_oklab_to_srgb(mut lab: [f64; 3]) -> (Color, bool) {
    // Hacky adjustment so that l=1.0 is close to pure white
    lab[0] *= 1.039;

    // Hacky adjustment for color balance (blue was oddly too bright)
    if lab[2] < 0.0 {
        lab[2] *= 0.8;
    }

    let l = lab[0] + 0.3963377774 * lab[1] + 0.2158037573 * lab[2];
    let m = lab[0] - 0.1055613458 * lab[1] - 0.0638541728 * lab[2];
    let s = lab[0] - 0.0894841775 * lab[1] - 1.2914855480 * lab[2];

    let l = l * l * l;
    let m = m * m * m;
    let s = s * s * s;

    let r = 4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s;
    let g = -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s;
    let b = -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s;

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
    // Note: greatest saturation available at val=0.75

    assert_eq!(oklab_hsv_to_srgb([0.0, 0.0, 0.0]), Some([0, 0, 0]));
    assert_eq!(
        oklab_hsv_to_srgb([0.0, 0.0, 1.0]),
        Some([65485, 65485, 65485])
    );
    assert_eq!(
        oklab_hsv_to_srgb([0.0, 0.175, 0.75]),
        Some([52629, 43619, 17165])
    );
    assert_eq!(
        oklab_hsv_to_srgb([1.0, 0.175, 0.75]),
        Some([52629, 43619, 17165])
    );
    assert_eq!(
        oklab_hsv_to_srgb([0.25, 0.175, 0.75]),
        Some([61232, 35563, 43801])
    );
}
