use crate::math::PartialClamp;

fn hue(mut h: f32, m1: f32, m2: f32) -> f32 {
    if h < 0.0 {
        h += 1.0;
    }
    if h > 1.0 {
        h -= 1.0;
    }

    if h < 1.0 / 6.0 {
        m1 + (m2 - m1) * h * 6.0
    } else if h < 3.0 / 6.0 {
        m2
    } else if h < 4.0 / 6.0 {
        m1 + (m2 - m1) * (2.0 / 3.0 - h) * 6.0
    } else {
        m1
    }
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        let Self { r, g, b, a } = self;
        [r, g, b, a]
    }
}

#[derive(Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    const fn raw(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub const TRANSPARENT: Self = Self::raw(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::raw(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::raw(1.0, 1.0, 1.0, 1.0);
    pub const RED: Self = Self::raw(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::raw(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::raw(0.0, 0.0, 1.0, 1.0);
}

impl Color {
    pub fn hex(color: u32) -> Self {
        let [b, g, r, a] = color.to_le_bytes();
        Self::new_srgba8(r, g, b, a)
    }

    pub fn new_srgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        use palette::{LinSrgba, Pixel, Srgba};
        let buffer = &[r, g, b, a];
        let raw = Srgba::from_raw(buffer);
        let raw_float: Srgba<f32> = raw.into_format();
        let lin: LinSrgba<f32> = raw_float.into_linear();

        Self {
            r: lin.color.red,
            g: lin.color.green,
            b: lin.color.blue,
            a: lin.alpha,
        }
    }

    pub fn is_transparent_black(&self) -> bool {
        self.r == 0.0 && self.g == 0.0 && self.b == 0.0 && self.a == 0.0
    }

    /// Returns color value specified by hue, saturation and lightness.
    /// HSL values are all in range [0..1], alpha will be set to 255.
    pub fn hsl(h: f32, s: f32, l: f32) -> Self {
        Self::hsla(h, s, l, 255)
    }

    /// Returns color value specified by hue, saturation and lightness and alpha.
    /// HSL values are all in range [0..1], alpha in range [0..255]
    pub fn hsla(h: f32, s: f32, l: f32, a: u8) -> Self {
        let mut h = h % 1.0;
        if h < 0.0 {
            h += 1.0;
        }
        let s = s.clamp(0.0, 1.0);
        let l = l.clamp(0.0, 1.0);

        let m2 = if l <= 0.5 {
            l * (1.0 + s)
        } else {
            l + s - l * s
        };
        let m1 = 2.0 * l - m2;
        Self {
            r: hue(h + 1.0 / 3.0, m1, m2).clamp(0.0, 1.0),
            g: hue(h, m1, m2).clamp(0.0, 1.0),
            b: hue(h - 1.0 / 3.0, m1, m2).clamp(0.0, 1.0),
            a: f32::from(a) / 255.0,
        }
    }

    pub fn premul(self) -> [f32; 4] {
        let Self { r, g, b, a } = self;
        [r * a, g * a, b * a, a]
    }
}
