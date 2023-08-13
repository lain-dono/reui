use palette::{encoding, Hsla, IntoColor, LinSrgb, LinSrgba, Srgb, Srgba};

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

impl From<Hsla<encoding::Linear<encoding::Srgb>, f32>> for Color {
    fn from(hsla: Hsla<encoding::Linear<encoding::Srgb>, f32>) -> Self {
        let linear: LinSrgba<f32> = hsla.into_color();
        Self::from(linear)
    }
}

impl From<LinSrgb<f32>> for Color {
    fn from(color: LinSrgb<f32>) -> Self {
        Self::new(color.red, color.green, color.blue, 1.0)
    }
}

impl From<LinSrgba<f32>> for Color {
    fn from(LinSrgba { color, alpha }: LinSrgba<f32>) -> Self {
        Self::new(color.red, color.green, color.blue, alpha)
    }
}

impl From<Srgb<f32>> for Color {
    fn from(color: Srgb<f32>) -> Self {
        Self::from(Srgba { color, alpha: 1.0 })
    }
}

impl From<Srgba<f32>> for Color {
    fn from(color: Srgba<f32>) -> Self {
        let linear: LinSrgba<f32> = color.into_linear();
        Self::from(linear)
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
    pub fn bgra(color: u32) -> Self {
        let [b, g, r, a] = color.to_le_bytes();
        Self::new_srgba8(r, g, b, a)
    }

    #[inline]
    pub fn new_srgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        let color: Srgba<f32> = Srgba::from([r, g, b, a]).into_format();
        Self::from(color)
    }

    /// Returns color value specified by hue, saturation and lightness and alpha.
    /// HSL values are all in range [0..1], alpha in range [0..1]
    pub fn hsla(hue: f32, saturation: f32, lightness: f32, alpha: f32) -> Self {
        let components = (hue, saturation, lightness, alpha);
        let hsla: Hsla<encoding::Linear<encoding::Srgb>, f32> = Hsla::from_components(components);
        Self::from(hsla)
    }
}
