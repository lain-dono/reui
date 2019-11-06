#[derive(Clone, Copy, Default)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl std::ops::Add<Self> for Offset {
    type Output = Self;
    fn add(self, v: Self) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }
}

impl std::ops::Sub<Self> for Offset {
    type Output = Self;
    fn sub(self, v: Self) -> Self {
        Self {
            x: self.x - v.x,
            y: self.y - v.y,
        }
    }
}

impl std::ops::Mul<f32> for Offset {
    type Output = Self;
    fn mul(self, f: f32) -> Self {
        Self {
            x: self.x * f,
            y: self.y * f,
        }
    }
}

impl std::ops::Div<f32> for Offset {
    type Output = Self;
    fn div(self, f: f32) -> Self {
        Self {
            x: self.x / f,
            y: self.y / f,
        }
    }
}

impl Into<[f32; 2]> for Offset {
    fn into(self) -> [f32; 2] { [self.x, self.y] }
}

impl Into<(f32, f32)> for Offset {
    fn into(self) -> (f32, f32) { (self.x, self.y) }
}

impl From<[f32; 2]> for Offset {
    fn from([x, y]: [f32; 2]) -> Self { Self { x, y } }
}

impl From<(f32, f32)> for Offset {
    fn from((x, y): (f32, f32)) -> Self { Self { x, y } }
}

impl Offset {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn approx_eq_eps(self, other: Self, epsilon: f32) -> bool {
        let x = (self.x - other.x).abs() < epsilon;
        let y = (self.y - other.y).abs() < epsilon;
        x && y
    }

    pub fn length(self) -> f32 {
        self.square_length().sqrt()
    }

    pub fn square_length(self) -> f32 {
        self.x * self.x + self.y * self.y
    }

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

    pub fn yx(self) -> Self {
        Self {
            y: self.x,
            x: self.y,
        }
    }
}

