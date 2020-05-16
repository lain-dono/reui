#[derive(Clone, Copy, Debug)]
pub struct Transform {
    pub re: f32,
    pub im: f32,
    pub tx: f32,
    pub ty: f32,
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
    pub fn apply_vector(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        [self.re * x - self.im * y, self.im * x + self.re * y]
    }

    #[inline]
    pub fn apply_inv(&self, [x, y]: [f32; 2]) -> [f32; 2] {
        let id = (self.re * self.re + self.im * self.im).recip();
        let [re, im] = [self.re * id, self.im * id];
        let [dx, dy] = [x - self.tx, y - self.ty];
        [re * dx + im * dy, re * dy - im * dx]
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

    #[inline]
    pub fn prepend(self, lhs: Self) -> Self {
        lhs * self
    }

    #[inline]
    pub fn append(self, rhs: Self) -> Self {
        self * rhs
    }

    #[inline]
    pub fn prepend_mut(&mut self, lhs: Self) {
        *self = lhs * *self;
    }

    #[inline]
    pub fn append_mut(&mut self, rhs: Self) {
        *self *= rhs;
    }
}
