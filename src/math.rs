use palette::{LinSrgba, Pixel, Srgb, Srgba};

pub trait PartialClamp {
    fn clamp(self, min: Self, max: Self) -> Self;
}

impl<T: std::cmp::PartialOrd> PartialClamp for T {
    fn clamp(self, min: Self, max: Self) -> Self {
        assert!(min <= max);
        let mut x = self;
        if x < min {
            x = min;
        }
        if x > max {
            x = max;
        }
        x
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct Color {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl Into<[f32; 4]> for Color {
    fn into(self) -> [f32; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

impl Into<[u8; 4]> for Color {
    fn into(self) -> [u8; 4] {
        [
            (self.red.clamp(0.0, 1.0) * 255.0) as u8,
            (self.green.clamp(0.0, 1.0) * 255.0) as u8,
            (self.blue.clamp(0.0, 1.0) * 255.0) as u8,
            (self.alpha.clamp(0.0, 1.0) * 255.0) as u8,
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

    const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
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
        #[inline(always)]
        fn channel(h: f32, m1: f32, m2: f32) -> f32 {
            let h = h.rem_euclid(1.0);

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

        let hue = hue.rem_euclid(1.0);
        let saturation = saturation.clamp(0.0, 1.0);
        let lightness = lightness.clamp(0.0, 1.0);

        let m2 = if lightness <= 0.5 {
            lightness * (1.0 + saturation)
        } else {
            lightness + saturation - lightness * saturation
        };

        let m1 = 2.0 * lightness - m2;

        let red = channel(hue + 1.0 / 3.0, m1, m2);
        let green = channel(hue, m1, m2);
        let blue = channel(hue - 1.0 / 3.0, m1, m2);

        let color = Srgb::new(red, green, blue);
        Self::from_srgba(Srgba { color, alpha })
    }
}

#[derive(Clone, Copy, Default)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

macro_rules! impl_op {
    (Neg::neg for $ty:ty) => {
        impl std::ops::Neg for $ty {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                (-self.x, -self.y).into()
            }
        }
    };

    ($name:ident :: $fn:ident($op:tt) $name_a:ident :: $fn_a:ident($op_a:tt) for $ty:ty) => {
        impl std::ops::$name<Self> for $ty {
            type Output = Self;
            #[inline] fn $fn(self, other: Self) -> Self {  Self { x: self.x $op other.x, y: self.y $op other.y } }
        }
        impl std::ops::$name_a<Self> for $ty {
            #[inline] fn $fn_a(&mut self, other: Self) { self.x $op_a other.x; self.y $op_a other.y; }
        }
    };

    ($arg:ty => $name:ident :: $fn:ident($op:tt) $name_a:ident :: $fn_a:ident($op_a:tt) for $ty:ty) => {
        impl std::ops::$name<$arg> for $ty {
            type Output = Self;
            #[inline] fn $fn(self, other: $arg) -> Self { Self { x: self.x $op other, y: self.y $op other } }
        }
        impl std::ops::$name_a<$arg> for $ty {
            #[inline] fn $fn_a(&mut self, other: $arg) { self.x $op_a other; self.y $op_a other; }
        }
    };
}

impl_op!(Neg::neg for Offset);
impl_op!(Add::add(+) AddAssign::add_assign(+=) for Offset);
impl_op!(Sub::sub(-) SubAssign::sub_assign(-=) for Offset);
impl_op!(f32 => Mul::mul(*) MulAssign::mul_assign(*=) for Offset);
impl_op!(f32 => Div::div(/) DivAssign::div_assign(/=) for Offset);

impl Into<[f32; 2]> for Offset {
    #[inline]
    fn into(self) -> [f32; 2] {
        [self.x, self.y]
    }
}

impl Into<(f32, f32)> for Offset {
    #[inline]
    fn into(self) -> (f32, f32) {
        (self.x, self.y)
    }
}

impl From<[f32; 2]> for Offset {
    #[inline]
    fn from([x, y]: [f32; 2]) -> Self {
        Self { x, y }
    }
}

impl From<(f32, f32)> for Offset {
    #[inline]
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

impl Offset {
    #[inline]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }
    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    #[inline]
    pub fn approx_eq_eps(self, other: Self, epsilon: f32) -> bool {
        let x = (self.x - other.x).abs() < epsilon;
        let y = (self.y - other.y).abs() < epsilon;
        x && y
    }

    #[inline]
    pub fn magnitude(self) -> f32 {
        self.magnitude_sq().sqrt()
    }

    #[inline]
    pub fn magnitude_sq(self) -> f32 {
        self.x.mul_add(self.x, self.y * self.y)
    }

    #[inline]
    pub fn normalize(self) -> Self {
        self / self.magnitude()
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x.mul_add(other.x, self.y * other.y)
    }

    #[inline]
    pub fn cross(self, other: Self) -> f32 {
        self.x.mul_add(other.y, -(other.x * self.y))
    }
}

#[derive(Clone, Copy, Default)]
pub struct Corners {
    pub tl: f32,
    pub tr: f32,
    pub br: f32,
    pub bl: f32,
}

impl Corners {
    pub fn all_same(radius: f32) -> Self {
        Self {
            tr: radius,
            tl: radius,
            br: radius,
            bl: radius,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Rect {
    pub min: Offset,
    pub max: Offset,
}

impl Rect {
    #[inline]
    pub const fn new(min: Offset, max: Offset) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self::new(Offset::new(left, top), Offset::new(right, bottom))
    }

    #[inline]
    pub fn from_center(center: Offset, width: f32, height: f32) -> Self {
        let pad = Offset::new(width / 2.0, height / 2.0);
        Self::new(center - pad, center + pad)
    }

    #[inline]
    pub fn from_ltwh(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self::from_ltrb(left, top, left + width, top + height)
    }

    #[inline]
    pub fn from_points(a: Offset, b: Offset) -> Self {
        Self::new(Offset::min(a, b), Offset::max(a, b))
    }

    #[inline]
    pub fn to_xywh(&self) -> [f32; 4] {
        [self.min.x, self.min.y, self.dx(), self.dy()]
    }

    #[inline]
    pub fn from_size(w: f32, h: f32) -> Self {
        Self::new(Offset::zero(), Offset::new(w, h))
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.x >= self.max.x || self.min.y >= self.max.y
    }

    #[inline]
    pub fn contains(&self, p: Offset) -> bool {
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y
    }

    #[inline]
    pub fn dx(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn dy(&self) -> f32 {
        self.max.y - self.min.y
    }

    #[inline]
    pub fn size(&self) -> Offset {
        Offset::new(self.dx(), self.dy())
    }

    #[inline]
    pub fn center(&self) -> Offset {
        self.min + self.size() / 2.0
    }

    #[inline]
    pub fn translate(&self, v: Offset) -> Self {
        Self::new(self.min + v, self.max + v)
    }

    #[inline]
    pub fn inflate(&self, delta: f32) -> Self {
        let delta = Offset::new(delta, delta);
        Self::new(self.min - delta, self.max + delta)
    }

    #[inline]
    pub fn deflate(&self, delta: f32) -> Self {
        let delta = Offset::new(delta, delta);
        Self::new(self.min + delta, self.max - delta)
    }

    #[inline]
    pub fn intersect(r: Self, s: Self) -> Self {
        Self::new(Offset::max(r.min, s.min), Offset::min(r.max, s.max))
    }

    #[inline]
    pub fn union(r: Self, s: Self) -> Self {
        Self::new(Offset::min(r.min, s.min), Offset::max(r.max, s.max))
    }

    #[inline]
    pub fn overlaps(r: Self, s: Self) -> bool {
        r.min.x <= s.max.x && s.min.x <= r.max.x && r.min.y <= s.max.y && s.min.y <= r.max.y
    }
}

#[derive(Clone, Copy)]
pub struct RRect {
    pub rect: Rect,
    pub radius: Corners,
}

impl RRect {
    pub fn from_rect_and_radius(rect: Rect, radius: f32) -> Self {
        let radius = Corners::all_same(radius);
        Self { rect, radius }
    }

    pub fn new(o: Offset, s: Offset, radius: f32) -> Self {
        Self::from_rect_and_radius(Rect::from_points(o, o + s), radius)
    }

    pub fn rect(&self) -> Rect {
        self.rect
    }

    pub fn dx(&self) -> f32 {
        self.rect.dx()
    }

    pub fn dy(&self) -> f32 {
        self.rect.dy()
    }

    pub fn inflate(self, v: f32) -> Self {
        Self {
            rect: self.rect.inflate(v),
            ..self
        }
    }

    pub fn deflate(self, v: f32) -> Self {
        Self {
            rect: self.rect.deflate(v),
            ..self
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub re: f32,
    pub im: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Into<[f32; 4]> for Transform {
    #[inline]
    fn into(self) -> [f32; 4] {
        [self.re, self.im, self.tx, self.ty]
    }
}

impl std::ops::Mul<Self> for Transform {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
        Self {
            re: other.re * self.re - other.im * self.im,
            im: other.re * self.im + other.im * self.re,

            tx: other.tx * self.re - other.ty * self.im + self.tx,
            ty: other.tx * self.im + other.ty * self.re + self.ty,
        }
    }
}

impl std::ops::MulAssign<Self> for Transform {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl Transform {
    pub const IDENTITY: Self = Self {
        re: 1.0,
        im: 0.0,
        tx: 0.0,
        ty: 0.0,
    };

    #[inline]
    pub fn identity() -> Self {
        Self::IDENTITY
    }

    #[inline]
    pub fn new(tx: f32, ty: f32, rotation: f32, scale: f32) -> Self {
        let re = rotation.cos() * scale;
        let im = -rotation.sin() * scale;
        Self { re, im, tx, ty }
    }

    #[inline]
    pub fn translation(tx: f32, ty: f32) -> Self {
        let (re, im) = (1.0, 0.0);
        Self { re, im, tx, ty }
    }

    #[inline]
    pub fn rotation(theta: f32) -> Self {
        let (sin, cos) = theta.sin_cos();
        let (re, im) = (cos, -sin);
        let (tx, ty) = (0.0, 0.0);
        Self { re, im, tx, ty }
    }

    #[inline]
    pub fn scale(factor: f32) -> Self {
        let (re, im, tx, ty) = (factor, 0.0, 0.0, 0.0);
        Self { re, im, tx, ty }
    }

    #[inline]
    pub fn apply(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        [
            self.re * x - self.im * y + self.tx,
            self.im * x + self.re * y + self.ty,
        ]
    }

    #[inline]
    pub fn apply_inv(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        let id = (self.re * self.re + self.im * self.im).recip();
        let [re, im] = [self.re * id, self.im * id];
        let [dx, dy] = [x - self.tx, y - self.ty];
        [re * dx + im * dy, re * dy - im * dx]
    }

    #[inline]
    pub fn apply_vector(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        [self.re * x - self.im * y, self.im * x + self.re * y]
    }

    #[inline]
    pub fn apply_inv_vector(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        let id = (self.re * self.re + self.im * self.im).recip();
        let [re, im] = [self.re * id, self.im * id];
        [re * x + im * y, re * y - im * x]
    }

    #[inline]
    pub fn inverse(&self) -> Self {
        let id = -(self.re * self.re + self.im * self.im).recip();
        Self {
            re: -self.re * id,
            im: self.im * id,
            tx: (self.im * self.ty + self.re * self.tx) * id,
            ty: (self.re * self.ty - self.im * self.tx) * id,
        }
    }
}
