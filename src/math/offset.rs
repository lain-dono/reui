#[derive(Clone, Copy, Default)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl std::ops::Neg for Offset {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        [-self.x, -self.y].into()
    }
}

macro_rules! impl_op {
    (0 $op:ident<$arg:ident>::$fn:ident($tt:tt) for $ty:ty) => {
        impl std::ops::$op<$arg> for $ty {
            type Output = Self;
            #[inline]
            fn $fn(self, v: Self) -> Self {
                Self { x: self.x $tt v.x, y: self.y $tt v.y }
            }
        }
    };

    (1 $op:ident<$arg:ident>::$fn:ident($tt:tt) for $ty:ty) => {
        impl std::ops::$op<$arg> for $ty {
            type Output = Self;
            #[inline]
            fn $fn(self, v: $arg) -> Self {
                Self { x: self.x $tt v, y: self.y $tt v }
            }
        }
    };

    (2 $op:ident<$arg:ident>::$fn:ident($tt:tt) for $ty:ty) => {
        impl std::ops::$op<$arg> for $ty {
            #[inline]
            fn $fn(&mut self, v: Self) {
                self.x $tt v.x;
                self.y $tt v.y;
            }
        }
    };

    (3 $op:ident<$arg:ident>::$fn:ident($tt:tt) for $ty:ty) => {
        impl std::ops::$op<$arg> for $ty {
            #[inline]
            fn $fn(&mut self, v: $arg) {
                self.x $tt v;
                self.y $tt v;
            }
        }
    };
}

impl_op!(0 Add<Self>::add(+) for Offset);
impl_op!(0 Sub<Self>::sub(-) for Offset);
impl_op!(0 Mul<Self>::mul(*) for Offset);
impl_op!(0 Div<Self>::div(/) for Offset);

impl_op!(1 Mul<f32>::mul(*) for Offset);
impl_op!(1 Div<f32>::div(/) for Offset);

impl_op!(2 AddAssign<Self>::add_assign(+=) for Offset);
impl_op!(2 SubAssign<Self>::sub_assign(-=) for Offset);
impl_op!(2 MulAssign<Self>::mul_assign(*=) for Offset);
impl_op!(2 DivAssign<Self>::div_assign(/=) for Offset);

impl_op!(3 MulAssign<f32>::mul_assign(*=) for Offset);
impl_op!(3 DivAssign<f32>::div_assign(/=) for Offset);

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
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    #[inline]
    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn approx_eq_eps(self, other: Self, epsilon: f32) -> bool {
        let x = (self.x - other.x).abs() < epsilon;
        let y = (self.y - other.y).abs() < epsilon;
        x && y
    }

    #[inline]
    pub fn length(self) -> f32 {
        self.square_length().sqrt()
    }

    #[inline]
    pub fn square_length(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn normalize(self) -> Self {
        self / self.length()
    }

    #[inline]
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    #[inline]
    pub fn cross(self, other: Self) -> f32 {
        self.x * other.y - self.y * other.x
    }

    #[inline]
    pub fn translate(self, x: f32, y: f32) -> Self {
        self + Self::new(x, y)
    }

    #[inline]
    pub fn scale(self, x: f32, y: f32) -> Self {
        self * Self::new(x, y)
    }
}
