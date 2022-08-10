use palette::{GammaSrgba, Hsla, IntoColor, LinSrgb, LinSrgba, Pixel, Srgb, Srgba};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl From<Color> for [f32; 4] {
    fn from(c: Color) -> [f32; 4] {
        [c.red, c.green, c.blue, c.alpha]
    }
}

impl From<Color> for [u8; 4] {
    fn from(c: Color) -> [u8; 4] {
        [
            (c.red * 255.0) as u8,
            (c.green * 255.0) as u8,
            (c.blue * 255.0) as u8,
            (c.alpha * 255.0) as u8,
        ]
    }
}

impl Color {
    pub const TRANSPARENT: Self = Self::new(0.0, 0.0, 0.0, 0.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);

    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    #[inline]
    pub fn hex(color: u32) -> Self {
        let [b, g, r, a] = color.to_le_bytes();
        Self::new_srgba8(r, g, b, a)
    }

    #[inline]
    pub fn new_srgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let buffer = &[r, g, b, a];
        Self::from_srgba(Srgba::from_raw(buffer).into_format())
    }

    #[inline]
    pub fn from_srgb(color: Srgb<f32>) -> Self {
        Self::from_srgba(Srgba { color, alpha: 1.0 })
    }

    #[inline]
    pub fn from_srgba(color: Srgba<f32>) -> Self {
        Self::from_linear(color.into_linear())
    }

    #[inline]
    pub fn from_linear(LinSrgba { color, alpha }: LinSrgba<f32>) -> Self {
        Self::new(color.red, color.green, color.blue, alpha)
    }

    /// Returns color value specified by hue, saturation and lightness and alpha.
    /// HSL values are all in range [0..1], alpha in range [0..1]
    pub fn hsla(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Self {
        use palette::encoding::{Linear, Srgb};

        let hsla: Hsla<Linear<Srgb>, f32> = Hsla::from_components((
            (hue.rem_euclid(1.0) * std::f32::consts::TAU).to_degrees(),
            saturation.clamp(0.0, 1.0),
            lightness.clamp(0.0, 1.0),
            alpha,
        ));
        Self::from_linear(hsla.into_color())
    }
}
