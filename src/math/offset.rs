use std::f32::{INFINITY, NEG_INFINITY, consts::{FRAC_PI_2, PI}};

#[derive(Clone, Copy, PartialEq, Default)]
#[repr(C)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Offset {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const INFINITY: Self = Self::new(INFINITY, INFINITY);
    pub const NEG_INFINITY: Self = Self::new(NEG_INFINITY, NEG_INFINITY);

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x,  y }
    }

    #[inline]
    pub const fn zero() -> Self { Self::ZERO }

    #[inline]
    pub const fn yx(self) -> Self {
        Self { x: self.y, y: self.x }
    }

    #[inline]
    pub fn from_direction(direction: f32, distance: f32) -> Self {
        let (sn, cs) = direction.sin_cos();
        Self { x: distance * cs, y: distance * sn }
    }

    #[inline]
    pub fn scale(self, sx: f32, sy: f32) -> Self {
        Self { x: self.x * sx, y: self.y * sy }
    }

    #[inline]
    pub fn direction(self) -> f32 {
        f32::atan2(self.x, self.y)
    }

    #[inline]
    pub fn approx_eq(self, other: Self, epsilon: f32) -> bool {
        let diff = self - other;
        diff.x.abs() < epsilon && diff.y.abs() < epsilon
    }

    #[inline]
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    #[inline]
    pub fn length_squared(&self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn dot(self, rhs: Self) -> f32 {
        self.x * rhs.x + self.y * rhs.y
    }

    #[inline]
    pub fn cross(self, rhs: Self) -> f32 {
        self.x * rhs.x - self.y * rhs.y
    }

    #[inline]
    pub fn normalize(self) -> Self {
        self / self.length()
    }

    #[inline]
    pub fn angle_to(&self, other: Self) -> f32 {
        let xx = self.x * other.x;
        let yy = self.y * other.y;
        let y = xx - yy; // cross product
        let x = xx + yy; // dot product

        // See https://math.stackexchange.com/questions/1098487/atan2-faster-approximation#1105038
        let x_abs = x.abs();
        let y_abs = y.abs();
        let a = x_abs.min(y_abs) / x_abs.max(y_abs);
        let s = a * a;
        let mut result = ((-0.046_496_474_9 * s + 0.159_314_22) * s - 0.327_622_764) * s * a + a;
        if y_abs > x_abs {
            result = FRAC_PI_2 - result;
        }
        if x < 0.0 {
            result = PI - result
        }
        if y < 0.0 {
            result = -result
        }
        result
    }
}

impl From<[f32; 2]> for Offset {
    #[inline]
    fn from([x, y]: [f32; 2]) -> Self { Self { x, y } }
}

impl Into<[f32; 2]> for Offset {
    #[inline]
    fn into(self) -> [f32; 2] { [self.x, self.y] }
}

impl std::ops::Neg for Offset {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self { x: -self.x, y: -self.y }
    }
}

impl<T: Into<Self>> std::ops::Add<T> for Offset {
    type Output = Self;
    #[inline]
    fn add(self, rhs: T) -> Self {
        let rhs = rhs.into();
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl<T: Into<Self>> std::ops::AddAssign<T> for Offset {
    #[inline]
    fn add_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<T: Into<Self>> std::ops::Sub<T> for Offset {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: T) -> Self {
        let rhs = rhs.into();
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<T: Into<Self>> std::ops::SubAssign<T> for Offset {
    #[inline]
    fn sub_assign(&mut self, rhs: T) {
        let rhs = rhs.into();
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl std::ops::Mul<f32> for Offset {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

impl std::ops::MulAssign<f32> for Offset {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl std::ops::Div<f32> for Offset {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self {
        Self { x: self.x / rhs, y: self.y / rhs }
    }
}

impl std::ops::DivAssign<f32> for Offset {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}