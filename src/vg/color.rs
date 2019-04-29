use super::utils::clampf;

fn hue(mut h: f32, m1: f32, m2: f32) -> f32 {
    if h < 0.0 { h += 1.0; }
    if h > 1.0 { h -= 1.0; }

    if h < 1.0/6.0 {
        m1 + (m2 - m1) * h * 6.0
    } else if h < 3.0/6.0 {
        m2
    } else if h < 4.0/6.0 {
        m1 + (m2 - m1) * (2.0/3.0 - h) * 6.0
    } else {
        m1
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn is_transparent_black(&self) -> bool {
        self.r == 0.0 &&
        self.g == 0.0 &&
        self.b == 0.0 &&
        self.a == 0.0
    }

    pub const fn new(color: u32) -> Self {
        let [b, g, r, a] = color.to_le_bytes();
        Self::rgba(r, g, b, a)
    }

    /// Returns a color value from red, green, blue values. Alpha will be set to 255 (1.0f).
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    /// Returns a color value from red, green, blue and alpha values.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: (r as f32) / 255.0,
            g: (g as f32) / 255.0,
            b: (b as f32) / 255.0,
            a: (a as f32) / 255.0,
        }
    }

    /// Returns a color value from red, green, blue values. Alpha will be set to 1.0f.
    pub fn rgbf(r: f32, g: f32, b: f32) -> Color {
        Self::rgbaf(r, g, b, 1.0)
    }

    /// Returns a color value from red, green, blue and alpha values.
    pub fn rgbaf(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Linearly interpolates from color c0 to c1, and returns resulting color value.
    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        let t = clampf(t, 0.0, 1.0);
        let oneminu = 1.0 - t;
        Self {
            r: a.r * oneminu + b.r * t,
            g: a.g * oneminu + b.g * t,
            b: a.b * oneminu + b.b * t,
            a: a.a * oneminu + b.a * t,
        }
    }

    /// Sets transparency of a color value.
    pub fn trans(mut self, a: u8) -> Self {
        self.a = f32::from(a) / 255.0;
        self
    }

    /// Sets transparency of a color value.
    pub fn transf(mut self, a: f32) -> Self {
        self.a = a;
        self
    }

    /// Returns color value specified by hue, saturation and lightness.
    /// HSL values are all in range [0..1], alpha will be set to 255.
    pub fn hsl(h: f32, s: f32, l: f32) -> Color {
        Self::hsla(h, s, l, 255)
    }

    /// Returns color value specified by hue, saturation and lightness and alpha.
    /// HSL values are all in range [0..1], alpha in range [0..255]
    pub fn hsla(h: f32, s: f32, l: f32, a: u8) -> Self {
        let mut h = h % 1.0;
        if h < 0.0 { h += 1.0; }
        let s = clampf(s, 0.0, 1.0);
        let l = clampf(l, 0.0, 1.0);

        let m2 = if l <= 0.5 { l * (1.0 + s) } else { l + s - l * s };
        let m1 = 2.0 * l - m2;
        Self {
            r: clampf(hue(h + 1.0/3.0, m1, m2), 0.0, 1.0),
            g: clampf(hue(h, m1, m2), 0.0, 1.0),
            b: clampf(hue(h - 1.0/3.0, m1, m2), 0.0, 1.0),
            a: f32::from(a) / 255.0,
        }
    }

    pub fn premul(self) -> [f32; 4] {
        let Self { r, g, b, a } = self;
        [r * a, g * a, b * a, a]
    }
}
