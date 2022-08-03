use crate::oklab::oklab_to_srgb;

pub type Color = [u16; 3];

pub struct ColorCone {
    pub max_saturation: f64,
    pub min_lightness: f64,
    pub max_lightness: f64,
}

impl ColorCone {
    pub fn hsv(&self, hue: f64, sat: f64, val: f64) -> Option<Color> {
        assert!(0.0 <= hue && hue <= 1.0);
        assert!(0.0 <= sat && sat <= 1.0);
        assert!(0.0 <= val && val <= 1.0);
        const RADS_PER_TURN: f64 = 2.0 * std::f64::consts::PI;
        fn interpolate(f: f64, start: f64, end: f64) -> f64 {
            start + f * (end - start)
        }
        let l = interpolate(val, self.min_lightness, self.max_lightness);
        let s = sat * self.max_saturation * val;
        let a = s * (RADS_PER_TURN * hue).sin();
        let b = s * (RADS_PER_TURN * hue).cos();
        let (color, is_clamped) = oklab_to_srgb([l, a, b]);
        if is_clamped {
            None
        } else {
            Some(color)
        }
    }
}

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
