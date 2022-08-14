use std::ops::{Add, Div, Mul, Neg, Rem, Sub};
use std::ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign};

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a.mul_add(1.0 - t, b * t)
}

macro_rules! impl_op {
    ($trait:ident<f32> for $dst:ident fn $fn:ident ($op:tt) -> $output:ident) => {
        impl $trait<f32> for $dst {
            type Output = Self;
            fn $fn(self, rhs: f32) -> Self::Output {
                Self::Output::new(self.x $op rhs, self.y $op rhs)
            }
        }
    };
    ($trait:ident<$rhs:ident> for $dst:ident fn $fn:ident ($op:tt) -> $output:ident) => {
        impl $trait<$rhs> for $dst {
            type Output = $output;
            fn $fn(self, rhs: $rhs) -> Self::Output {
                Self::Output::new(self.x $op rhs.x, self.y $op rhs.y)
            }
        }
    };
}

macro_rules! impl_assign {
    ($trait:ident<f32> for $dst:ident fn $fn:ident ($op:tt)) => {
        impl $trait<f32> for $dst {
            fn $fn(&mut self, rhs: f32) {
                self.x $op rhs;
                self.y $op rhs;
            }
        }
    };
    ($trait:ident<$rhs:ident> for $dst:ident fn $fn:ident ($op:tt)) => {
        impl $trait<$rhs> for $dst {
            fn $fn(&mut self, rhs: $rhs) {
                self.x $op rhs.x;
                self.y $op rhs.y;
            }
        }
    };
}

macro_rules! impl_conv {
    ($dst:ident) => {
        impl From<(f32, f32)> for $dst {
            fn from((x, y): (f32, f32)) -> Self {
                Self { x, y }
            }
        }
        impl From<[f32; 2]> for $dst {
            fn from([x, y]: [f32; 2]) -> Self {
                Self { x, y }
            }
        }
        impl From<$dst> for (f32, f32) {
            fn from($dst { x, y }: $dst) -> (f32, f32) {
                (x, y)
            }
        }
        impl From<$dst> for [f32; 2] {
            fn from($dst { x, y }: $dst) -> [f32; 2] {
                [x, y]
            }
        }
    };
}

impl Neg for Offset {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        (-self.x, -self.y).into()
    }
}

impl_conv!(Offset);
impl_op!(Add<Self> for Offset fn add(+) -> Self);
impl_op!(Sub<Self> for Offset fn sub(-) -> Self);
impl_op!(Mul<f32> for Offset fn mul(*) -> Self);
impl_op!(Div<f32> for Offset fn div(/) -> Self);
impl_op!(Rem<f32> for Offset fn rem(%) -> Self);

impl_assign!(AddAssign<Self> for Offset fn add_assign(+=));
impl_assign!(SubAssign<Self> for Offset fn sub_assign(-=));
impl_assign!(MulAssign<f32> for Offset fn mul_assign(*=));
impl_assign!(DivAssign<f32> for Offset fn div_assign(/=));
impl_assign!(RemAssign<f32> for Offset fn rem_assign(%=));

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Offset {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    pub const fn infinity() -> Self {
        Self::new(f32::INFINITY, f32::INFINITY)
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
    pub fn cross(self, other: Self) -> f32 {
        // sx * oy + -(ox * sy)
        self.x.mul_add(other.y, -(other.x * self.y))
    }

    #[inline]
    pub fn floor(self) -> Self {
        self.map(f32::floor)
    }

    #[inline]
    pub fn ceil(self) -> Self {
        self.map(f32::ceil)
    }

    #[inline]
    pub fn round(self) -> Self {
        self.map(f32::round)
    }

    pub fn magnitude(self) -> f32 {
        self.x.hypot(self.y)
    }

    pub fn magnitude_sq(self) -> f32 {
        self.x.mul_add(self.x, self.y * self.y)
    }

    pub fn scale(self, x: f32, y: f32) -> Self {
        Self::new(self.x * x, self.y * y)
    }

    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        Self {
            x: lerp(a.x, b.x, t),
            y: lerp(a.y, b.y, t),
        }
    }

    pub fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }

    #[inline(always)]
    fn map(self, map: impl Fn(f32) -> f32) -> Self {
        Self::new(map(self.x), map(self.y))
    }
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct Rounding {
    /// Radius of the rounding of the North-West (left top) corner.
    pub nw: f32,
    /// Radius of the rounding of the North-East (right top) corner.
    pub ne: f32,
    /// Radius of the rounding of the South-West (left bottom) corner.
    pub sw: f32,
    /// Radius of the rounding of the South-East (right bottom) corner.
    pub se: f32,
}

impl Rounding {
    pub const fn new(nw: f32, ne: f32, sw: f32, se: f32) -> Self {
        Self { nw, ne, sw, se }
    }

    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    pub const fn top(radius: f32) -> Self {
        Self::new(radius, radius, 0.0, 0.0)
    }

    pub const fn bottom(radius: f32) -> Self {
        Self::new(0.0, 0.0, radius, radius)
    }

    pub const fn left(radius: f32) -> Self {
        Self::new(0.0, radius, 0.0, radius)
    }

    pub const fn right(radius: f32) -> Self {
        Self::new(radius, 0.0, radius, 0.0)
    }

    pub const fn same(radius: f32) -> Self {
        Self {
            ne: radius,
            nw: radius,
            sw: radius,
            se: radius,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Default, Debug)]
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
    pub fn from_oval(cx: f32, cy: f32, rx: f32, ry: f32) -> Self {
        let center = Offset::new(cx, cy);
        let pad = Offset::new(rx, ry);
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
    pub fn translate(&self, offset: Offset) -> Self {
        Self::new(self.min + offset, self.max + offset)
    }

    #[inline]
    pub fn shift(&self, x: f32, y: f32) -> Self {
        let offset = Offset::new(x, y);
        Self::new(self.min + offset, self.max + offset)
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Transform {
    pub re: f32,
    pub im: f32,
    pub tx: f32,
    pub ty: f32,
}

impl Default for Transform {
    #[inline]
    fn default() -> Self {
        Self::identity()
    }
}

impl From<Transform> for [f32; 4] {
    fn from(Transform { re, im, tx, ty }: Transform) -> [f32; 4] {
        [re, im, tx, ty]
    }
}

impl Mul<Self> for Transform {
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

impl MulAssign<Self> for Transform {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other;
    }
}

impl Transform {
    #[inline]
    pub const fn identity() -> Self {
        Self::new(1.0, 0.0, 0.0, 0.0)
    }

    #[inline]
    pub const fn new(re: f32, im: f32, tx: f32, ty: f32) -> Self {
        Self { re, im, tx, ty }
    }

    #[inline]
    pub fn compose(tx: f32, ty: f32, rotation: f32, scale: f32) -> Self {
        let (im, re) = (rotation.sin() * scale, rotation.cos() * scale);
        Self { re, im, tx, ty }
    }

    #[inline]
    pub fn translation(tx: f32, ty: f32) -> Self {
        Self::new(1.0, 0.0, tx, ty)
    }

    #[inline]
    pub fn rotation(theta: f32) -> Self {
        Self::new(theta.cos(), theta.sin(), 0.0, 0.0)
    }

    #[inline]
    pub fn scale(factor: f32) -> Self {
        Self::new(factor, 0.0, 0.0, 0.0)
    }

    #[inline]
    pub fn apply(&self, Offset { x, y }: Offset) -> Offset {
        Offset {
            x: self.re * x - self.im * y + self.tx,
            y: self.im * x + self.re * y + self.ty,
        }
    }

    #[inline]
    pub fn apply_inv(&self, Offset { x, y }: Offset) -> Offset {
        let id = (self.re * self.re + self.im * self.im).recip();
        let [re, im] = [self.re * id, self.im * id];
        let [dx, dy] = [x - self.tx, y - self.ty];
        Offset::new(re * dx + im * dy, re * dy - im * dx)
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

pub trait Bezier32:
    Copy + Add<Output = Self> + Mul<f32, Output = Self> + Div<f32, Output = Self>
{
    #[inline]
    fn conic(p: [Self; 3], w: f32, t: f32) -> Self {
        let h = 1.0 - t;
        let [a, b, c] = [h * h, h * t * 2.0 * w, t * t];
        (p[0] * a + p[1] * b + p[2] * c) / (a + b + c)
    }

    #[inline]
    fn bezier2(p: [Self; 3], t: f32) -> Self {
        let h = 1.0 - t;
        let [a, b, c] = [h * h, h * t * 2.0, t * t];
        p[0] * a + p[1] * b + p[2] * c
    }

    #[inline]
    fn rational2(p: [Self; 3], w: [f32; 3], t: f32) -> Self {
        let h = 1.0 - t;
        let [a, b, c] = [h * h, h * t * 2.0, t * t];
        let [a, b, c] = [a * w[0], b * w[1], c * w[2]];
        (p[0] * a + p[1] * b + p[2] * c) / (a + b + c)
    }

    #[inline]
    fn bezier3(p: [Self; 4], t: f32) -> Self {
        let [a, b, c, d] = Self::bezier_args3(t);
        p[0] * a + p[1] * b + p[2] * c + p[3] * d
    }

    #[inline]
    fn rational3(p: [Self; 4], w: [f32; 4], t: f32) -> Self {
        let [a, b, c, d] = Self::bezier_args3(t);
        let [a, b, c, d] = [a * w[0], b * w[1], c * w[2], d * w[3]];
        (p[0] * a + p[1] * b + p[2] * c + p[3] * d) / (a + b + c + d)
    }

    #[inline(always)]
    fn bezier_args3(t: f32) -> [f32; 4] {
        let h = 1.0 - t;

        let tt = t * t;
        let hh = h * h;

        let a = hh * h;
        let b = hh * t;
        let c = tt * h;
        let d = tt * t;

        [a, b * 3.0, c * 3.0, d]
    }
}

impl<T> Bezier32 for T where
    T: Copy + Add<Output = Self> + Mul<f32, Output = Self> + Div<f32, Output = Self>
{
}
