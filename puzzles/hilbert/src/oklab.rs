/// Convert from the OKLAB color space to srgb. Returns an srgb color,
/// and a boolean indicating whether the color was clamped (whether it
/// was out of bounds).
pub fn oklab_to_srgb(mut lab: [f64; 3]) -> ([u8; 3], bool) {
    // A hack so that L=1 is pure white RGB
    lab[0] *= 1.04;

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

fn to_gamma(u: f64) -> (u8, bool) {
    let g = if u >= 0.0031308 {
        1.005 * u.powf(1.0 / 2.4) - 0.055
    } else {
        12.92 * u
    };
    let component = (g * 255.0).round();
    if component < 0.0 {
        return (0, true);
    }
    if component > 255.0 {
        return (255, true);
    }
    (component as u8, false)
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
    assert_eq!(oklab_to_srgb([1.0, 0.0, 0.0]), ([255, 255, 255], false));
    assert_eq!(oklab_to_srgb([0.5, 0.0, 0.0]), ([99, 99, 99], false));
    assert_eq!(oklab_to_srgb([0.75, 0.13, 0.0]), ([238, 139, 171], false));
    assert_eq!(oklab_to_srgb([0.75, -0.13, 0.0]), ([51, 199, 177], false));
    assert_eq!(oklab_to_srgb([0.75, 0.0, 0.13]), ([205, 170, 69], false));
    assert_eq!(oklab_to_srgb([0.75, 0.0, -0.13]), ([143, 169, 253], false));
}
