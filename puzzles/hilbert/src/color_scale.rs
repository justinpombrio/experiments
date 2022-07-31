use crate::oklab::oklab_to_srgb;

pub type Color = [u16; 3];

pub struct ColorScaleConfig {
    pub hue_range: (f64, f64),
    pub shade_range: (f64, f64),
    pub min_shade: f64,
    pub saturation: f64,
    pub lightness: f64,
}

pub fn color_scale(f: f64, config: ColorScaleConfig) -> (Color, bool) {
    const RADS_PER_TURN: f64 = 2.0 * std::f64::consts::PI;

    fn interpolate(f: f64, (start, end): (f64, f64)) -> f64 {
        start + f * (end - start)
    }

    let angle = interpolate(f, config.hue_range);
    let shade = interpolate(f, config.shade_range);
    let shade = ((shade % 2.0) - 1.0).abs();
    let shade = shade * (1.0 - config.min_shade) + config.min_shade;
    let saturation = shade * config.saturation;
    let l = shade * config.lightness;
    let a = saturation * (RADS_PER_TURN * angle).sin();
    let b = saturation * (RADS_PER_TURN * angle).cos();
    oklab_to_srgb([l, a, b])
}
