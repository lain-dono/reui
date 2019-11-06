mod color;
pub mod transform;

pub use self::{
    color::Color,
    transform::Transform,
};

impl self::transform::Transform {
    pub fn transform_point(&self, p: Point) -> Point {
        self.apply(p.into()).into()
    }
}

//pub type Point = euclid::Point2D<f32, UnknownUnit>;
//pub type Vector = euclid::Vector2D<f32, UnknownUnit>;

#[inline]
pub fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::new(
        Point { x, y },
        Vector::new(w, h),
    )
}

#[inline]
pub fn vec2(x: f32, y: f32) -> Vector {
    Vector { x, y }
}

#[inline]
pub fn point2(x: f32, y: f32) -> Point {
    Point { x, y }
}

#[derive(Clone, Copy, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Default)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl std::ops::Add<Vector> for Point {
    type Output = Point;
    fn add(self, v: Vector) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }
}

impl std::ops::Sub<Vector> for Point {
    type Output = Point;
    fn sub(self, v: Vector) -> Self {
        Self {
            x: self.x - v.x,
            y: self.y - v.y,
        }
    }
}

impl std::ops::Sub<Point> for Point {
    type Output = Vector;
    fn sub(self, p: Self) -> Vector {
        Vector {
            x: self.x - p.x,
            y: self.y - p.y,
        }
    }
}

impl std::ops::Add<Vector> for Vector {
    type Output = Vector;
    fn add(self, v: Self) -> Self {
        Self {
            x: self.x + v.x,
            y: self.y + v.y,
        }
    }
}

impl std::ops::Sub<Vector> for Vector {
    type Output = Vector;
    fn sub(self, v: Self) -> Self {
        Self {
            x: self.x - v.x,
            y: self.y - v.y,
        }
    }
}

impl std::ops::Mul<f32> for Vector {
    type Output = Vector;
    fn mul(self, f: f32) -> Self {
        Self {
            x: self.x * f,
            y: self.y * f,
        }
    }
}

impl std::ops::Div<f32> for Vector {
    type Output = Vector;
    fn div(self, f: f32) -> Self {
        Self {
            x: self.x / f,
            y: self.y / f,
        }
    }
}

impl Into<[f32; 2]> for Vector {
    fn into(self) -> [f32; 2] { [self.x, self.y] }
}

impl Into<[f32; 2]> for Point {
    fn into(self) -> [f32; 2] { [self.x, self.y] }
}

impl From<[f32; 2]> for Vector {
    fn from([x, y]: [f32; 2]) -> Self { Self { x, y } }
}

impl From<[f32; 2]> for Point {
    fn from([x, y]: [f32; 2]) -> Self { Self { x, y } }
}


impl Into<(f32, f32)> for Vector {
    fn into(self) -> (f32, f32) { (self.x, self.y) }
}

impl Into<(f32, f32)> for Point {
    fn into(self) -> (f32, f32) { (self.x, self.y) }
}

impl From<(f32, f32)> for Vector {
    fn from((x, y): (f32, f32)) -> Self { Self { x, y } }
}

impl From<(f32, f32)> for Point {
    fn from((x, y): (f32, f32)) -> Self { Self { x, y } }
}

impl Point {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn to_vector(self) -> Vector {
        Vector { x: self.x, y: self.y }
    }

    pub fn approx_eq_eps(self, other: Self, epsilon: f32) -> bool {
        let x = (self.x - other.x).abs() < epsilon;
        let y = (self.y - other.y).abs() < epsilon;
        x && y
    }
}

impl Vector {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn length(&self) -> f32 {
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

#[derive(Clone, Copy)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    pub fn new(min: Point, size: Vector) -> Self {
        Self { min, max: min + size }
    }

    pub fn to_xywh(&self) -> [f32; 4] {
        [
            self.min.x,
            self.min.y,
            self.dx(),
            self.dy(),
        ]
    }

    pub fn from_size(w: f32, h: f32) -> Self {
        let size = vec2(w, h);
        Self { min: point2(0.0, 0.0), max: point2(0.0, 0.0) + size }
    }

    pub fn dx(&self) -> f32 { self.max.x - self.min.x }
    pub fn dy(&self) -> f32 { self.max.y - self.min.y }

    pub fn size(&self) -> Vector {
        Vector::new(self.dx(), self.dy())
    }

    pub fn center(&self) -> Point {
        self.min + self.size() / 2.0
    }

    pub fn translate(&self, v: Vector) -> Self {
        Self {
            min: self.min + v,
            max: self.max + v,
        }
    }

    pub fn inflate(&self, delta: f32) -> Self {
        Self {
            min: point2(self.min.x - delta, self.min.y - delta),
            max: point2(self.max.x + delta, self.max.y + delta),
        }
    }

    pub fn deflate(&self, delta: f32) -> Self {
        Self {
            min: point2(self.min.x + delta, self.min.y + delta),
            max: point2(self.max.x - delta, self.max.y - delta),
        }
    }
}