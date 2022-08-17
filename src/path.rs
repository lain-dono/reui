use crate::{Offset, Rect, Rounding, Transform};

// Length proportional to radius of a cubic bezier handle for 90deg arcs.
const KAPPA90: f32 = 0.552_284_8; // 0.5522847493

/// The fill rule used when filling paths: `EvenOdd`, `NonZero`.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

impl Default for FillRule {
    fn default() -> Self {
        Self::NonZero
    }
}

/// Used to specify Solid/Hole when adding shapes to a path.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Solidity {
    Solid,
    Hole,
}

impl Default for Solidity {
    fn default() -> Self {
        Self::Solid
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
enum Raw {
    MoveTo,
    LineTo,
    QuadTo,
    CubicTo,
    Solid,
    Hole,
    Close,
}

impl Raw {
    fn num_points(self) -> usize {
        match self {
            Self::MoveTo | Self::LineTo => 1,
            Self::QuadTo => 2,
            Self::CubicTo => 3,
            Self::Solid | Self::Hole | Self::Close => 0,
        }
    }
}

/// Path drawing command.
#[derive(Clone, Copy)]
pub enum Command {
    MoveTo(Offset),
    LineTo(Offset),
    QuadTo(Offset, Offset),
    CubicTo(Offset, Offset, Offset),
    Solid,
    Hole,
    Close,
}

impl Command {
    #[inline]
    fn to_raw(self, array: &mut [Offset; 3]) -> (Raw, usize) {
        match self {
            Command::MoveTo(p) => {
                array[0] = p;
                (Raw::MoveTo, 1)
            }
            Command::LineTo(p) => {
                array[0] = p;
                (Raw::LineTo, 1)
            }
            Command::QuadTo(p1, p2) => {
                array[0] = p1;
                array[1] = p2;
                (Raw::QuadTo, 2)
            }
            Command::CubicTo(p1, p2, p3) => {
                array[0] = p1;
                array[1] = p2;
                array[2] = p3;
                (Raw::CubicTo, 3)
            }
            Command::Solid => (Raw::Solid, 0),
            Command::Hole => (Raw::Hole, 0),
            Command::Close => (Raw::Close, 0),
        }
    }
}

pub struct PathIter<'a> {
    index: std::slice::Iter<'a, Raw>,
    coord: &'a [Offset],
}

impl<'a> PathIter<'a> {
    pub(crate) fn transform(self, transform: Transform) -> PathTransformIter<'a> {
        PathTransformIter {
            transform,
            index: self.index,
            coord: self.coord,
        }
    }
}

impl<'a> Iterator for PathIter<'a> {
    type Item = Command;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.index.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&raw) = self.index.next() {
            let (coords, next) = self.coord.split_at(raw.num_points());
            self.coord = next;
            Some(match raw {
                Raw::MoveTo => Command::MoveTo(coords[0]),
                Raw::LineTo => Command::LineTo(coords[0]),
                Raw::QuadTo => Command::QuadTo(coords[0], coords[1]),
                Raw::CubicTo => Command::CubicTo(coords[0], coords[1], coords[2]),
                Raw::Solid => Command::Solid,
                Raw::Hole => Command::Hole,
                Raw::Close => Command::Close,
            })
        } else {
            None
        }
    }
}

pub struct PathTransformIter<'a> {
    transform: Transform,
    index: std::slice::Iter<'a, Raw>,
    coord: &'a [Offset],
}

impl<'a> Iterator for PathTransformIter<'a> {
    type Item = Command;

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.index.size_hint()
    }

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&raw) = self.index.next() {
            let (coords, next) = self.coord.split_at(raw.num_points());
            self.coord = next;
            Some(match raw {
                Raw::MoveTo => Command::MoveTo(self.transform.apply(coords[0])),
                Raw::LineTo => Command::LineTo(self.transform.apply(coords[0])),
                Raw::QuadTo => Command::QuadTo(
                    self.transform.apply(coords[0]),
                    self.transform.apply(coords[1]),
                ),
                Raw::CubicTo => Command::CubicTo(
                    self.transform.apply(coords[0]),
                    self.transform.apply(coords[1]),
                    self.transform.apply(coords[2]),
                ),
                Raw::Solid => Command::Solid,
                Raw::Hole => Command::Hole,
                Raw::Close => Command::Close,
            })
        } else {
            None
        }
    }
}

/// Collection of drawing commands.
#[derive(Default, Clone)]
pub struct Path {
    index: Vec<Raw>,
    coord: Vec<Offset>,
}

impl<'a> IntoIterator for &'a Path {
    type Item = Command;
    type IntoIter = PathIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        PathIter {
            index: self.index.iter(),
            coord: &self.coord,
        }
    }
}

impl std::iter::Extend<Command> for Path {
    fn extend<T: IntoIterator<Item = Command>>(&mut self, iter: T) {
        let mut coord = [Offset::zero(); 3];
        for cmd in iter {
            let (cmd, len) = cmd.to_raw(&mut coord);
            self.index.push(cmd);
            self.coord.extend_from_slice(&coord[..len]);
        }
    }
}

impl<'a> std::iter::Extend<&'a Command> for Path {
    fn extend<T: IntoIterator<Item = &'a Command>>(&mut self, iter: T) {
        let mut coord = [Offset::zero(); 3];
        for cmd in iter {
            let (cmd, len) = cmd.to_raw(&mut coord);
            self.index.push(cmd);
            self.coord.extend_from_slice(&coord[..len]);
        }
    }
}

impl Path {
    pub(crate) fn iter(&self) -> PathIter {
        PathIter {
            index: self.index.iter(),
            coord: &self.coord,
        }
    }

    pub(crate) fn transform_inplace(&mut self, tx: f32, ty: f32, rotation: f32, scale: f32) {
        let t = Transform::compose(tx, ty, rotation, scale);
        for coord in &mut self.coord {
            *coord = t.apply(*coord);
        }
    }

    /// Returns a copy of the [`Path`] with all the segments of every sub-path transformed by the given matrix.
    pub(crate) fn transform_iter(&self, transform: Transform) -> PathTransformIter {
        PathTransformIter {
            transform,
            index: self.index.iter(),
            coord: &self.coord,
        }
    }

    pub(crate) fn extend_with_path(&mut self, other: &Self) {
        self.index.extend_from_slice(&other.index);
        self.coord.extend_from_slice(&other.coord);
    }
}

impl Path {
    /// Creates a new [`Path`].
    pub const fn new() -> Self {
        Self {
            index: Vec::new(),
            coord: Vec::new(),
        }
    }

    /// Clears the [`Path`], removing all drawing commands.
    pub fn clear(&mut self) {
        self.index.clear();
        self.coord.clear();
    }

    /// Returns `true` if the [`Path`] does not contain any drawing commands.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Moves the starting point of a new sub-path to the given [`Offset`].
    pub fn move_to(&mut self, p0: Offset) {
        self.index.push(Raw::MoveTo);
        self.coord.push(p0);
    }

    /// Connects the last point in the [`Offset`] to the given Point with a straight line.
    pub fn line_to(&mut self, p1: Offset) {
        self.index.push(Raw::LineTo);
        self.coord.push(p1);
    }

    /// Adds a quadratic Bézier curve to the [`Path`] given its control point and its end point.
    pub fn quad_to(&mut self, p1: Offset, p2: Offset) {
        self.index.push(Raw::QuadTo);
        self.coord.extend([p1, p2]);
    }

    /// Adds a cubic Bézier curve to the [`Path`] given its two control points and its end point.
    pub fn cubic_to(&mut self, p1: Offset, p2: Offset, p3: Offset) {
        self.index.push(Raw::CubicTo);
        self.coord.extend([p1, p2, p3]);
    }

    /// Closes the current sub-path in the [`Path`] with a straight line to the starting point.
    pub fn close(&mut self) {
        self.index.push(Raw::Close);
    }

    /// Sets the current sub-path winding, see [`Solidity`]
    pub fn solidity(&mut self, solidity: Solidity) {
        match solidity {
            Solidity::Solid => self.index.push(Raw::Solid),
            Solidity::Hole => self.index.push(Raw::Hole),
        }
    }
}

impl Path {
    /*
    /// Adds a new sub-path with one arc segment that consists of the arc that follows the edge of the oval
    /// bounded by the given rectangle,
    /// from startAngle radians around the oval up to startAngle + sweepAngle radians around the oval,
    /// with zero radians being the point on the right hand side of the oval that crosses the horizontal line
    /// that intersects the center of the rectangle and with positive angles going clockwise around the oval.
    pub fn add_arc(Rect oval, double startAngle, double sweepAngle) -> void
    */

    pub fn line(&mut self, p0: Offset, p1: Offset) {
        self.move_to(p0);
        self.line_to(p1);
    }

    pub fn polyline(&mut self, points: &[Offset], close: bool) {
        let mut iter = points.iter();
        if let Some(&p0) = iter.next() {
            self.move_to(p0);
            for &p in iter {
                self.line_to(p);
            }
            if close {
                self.close();
            }
        }
    }

    /// Adds a new sub-path that consists of a curve that forms the ellipse that fills the given rectangle.
    pub fn oval(&mut self, rect: Rect) {
        self.ellipse(rect.center(), rect.size() / 2.0);
    }

    /// Adds a circle to the [`Path`] given its center coordinate and its radius.
    pub fn circle(&mut self, center: Offset, radius: f32) {
        self.ellipse(center, Offset::new(radius, radius));
    }

    /// Adds [`Rect`] to the [`Path`].
    pub fn rect(&mut self, rect: Rect) {
        let Rect { min, max } = rect;
        self.index.extend([
            Raw::MoveTo,
            Raw::LineTo,
            Raw::LineTo,
            Raw::LineTo,
            Raw::Close,
        ]);
        self.coord.extend([
            Offset::new(min.x, min.y),
            Offset::new(min.x, max.y),
            Offset::new(max.x, max.y),
            Offset::new(max.x, min.y),
        ]);
    }

    /// Adds [`Rect`] with [`Rounding`] to the [`Path`].
    pub fn rrect(&mut self, rect: Rect, radius: Rounding) {
        let Rect { min, max } = rect;
        let kappa = 1.0 - KAPPA90;

        let sw = Offset::new(radius.sw, radius.sw);
        let se = Offset::new(radius.se, radius.se);
        let ne = Offset::new(radius.ne, radius.ne);
        let nw = Offset::new(radius.nw, radius.nw);

        self.move_to(Offset::new(min.x, min.y + nw.y));

        self.line_to(Offset::new(min.x, max.y - sw.y));

        if radius.sw > 0.0 {
            self.cubic_to(
                Offset::new(min.x, max.y - sw.y * kappa),
                Offset::new(min.x + sw.x * kappa, max.y),
                Offset::new(min.x + sw.x, max.y),
            );
        }

        self.line_to(Offset::new(max.x - se.x, max.y));
        if radius.se > 0.0 {
            self.cubic_to(
                Offset::new(max.x - se.x * kappa, max.y),
                Offset::new(max.x, max.y - se.y * kappa),
                Offset::new(max.x, max.y - se.y),
            );
        }

        self.line_to(Offset::new(max.x, min.y + ne.y));
        if radius.ne > 0.0 {
            self.cubic_to(
                Offset::new(max.x, min.y + ne.y * kappa),
                Offset::new(max.x - ne.x * kappa, min.y),
                Offset::new(max.x - ne.x, min.y),
            );
        }

        self.line_to(Offset::new(min.x + nw.x, min.y));
        if radius.nw > 0.0 {
            self.cubic_to(
                Offset::new(min.x + nw.x * kappa, min.y),
                Offset::new(min.x, min.y + nw.y * kappa),
                Offset::new(min.x, min.y + nw.y),
            );
        }

        self.close();
    }

    #[inline]
    fn ellipse(&mut self, center: Offset, radius: Offset) {
        let kappa = radius * KAPPA90;
        let (positive_radius, positive_kappa) = (center + radius, center + kappa);
        let (negative_radius, negative_kappa) = (center - radius, center - kappa);

        self.index.extend([
            Raw::MoveTo,
            Raw::CubicTo,
            Raw::CubicTo,
            Raw::CubicTo,
            Raw::CubicTo,
            Raw::Close,
        ]);

        self.coord.extend([
            Offset::new(negative_radius.x, center.y),         // move
            Offset::new(negative_radius.x, positive_kappa.y), // curve
            Offset::new(negative_kappa.x, positive_radius.y),
            Offset::new(center.x, positive_radius.y),
            Offset::new(positive_kappa.x, positive_radius.y), // curve
            Offset::new(positive_radius.x, positive_kappa.y),
            Offset::new(positive_radius.x, center.y),
            Offset::new(positive_radius.x, negative_kappa.y), // curve
            Offset::new(positive_kappa.x, negative_radius.y),
            Offset::new(center.x, negative_radius.y),
            Offset::new(negative_kappa.x, negative_radius.y), // curve
            Offset::new(negative_radius.x, negative_kappa.y),
            Offset::new(negative_radius.x, center.y),
        ]);
    }

    /*
    /// Appends up to four conic curves weighted to describe an oval of radius and rotated by rotation. [...]
    pub fn relative_arc_to_point(Offset arcEndDelta, { Radius radius: Radius.zero, double rotation: 0.0, bool largeArc: false, bool clockwise: true }) -> void
    /// Adds a bezier segment that curves from the current point to the point at the offset (x2,y2) from the current point, using the control point at the offset (x1,y1) from the current point and the weight w. If the weight is greater than 1, then the curve is a hyperbola; if the weight equals 1, it's a parabola; and if it is less than 1, it is an ellipse.
    pub fn relative_conic_to(double x1, double y1, double x2, double y2, double w) -> void
    /// Adds a cubic bezier segment that curves from the current point to the point at the offset (x3,y3) from the current point, using the control points at the offsets (x1,y1) and (x2,y2) from the current point.
    pub fn relative_cubic_to(double x1, double y1, double x2, double y2, double x3, double y3) -> void
    /// Adds a quadratic bezier segment that curves from the current point to the point at the offset (x2,y2) from the current point, using the control point at the offset (x1,y1) from the current point.
    pub fn relative_quadratic_bezier_to(double x1, double y1, double x2, double y2) -> void
    */

    /// Creates new circle arc shaped sub-path. The arc center is at cx,cy, the arc radius is r,
    /// and the arc is drawn from angle a0 to a1, and swept in direction dir (Winding)
    /// Angles are specified in radians.
    pub fn arc(&mut self, center: Offset, radius: f32, a0: f32, a1: f32, dir: Solidity) {
        use std::f32::consts::{FRAC_PI_2, TAU};

        // Clamp angles
        let mut da = a1 - a0;
        if let Solidity::Hole = dir {
            if da.abs() >= TAU {
                da = TAU;
            } else {
                while da < 0.0 {
                    da += TAU;
                }
            }
        } else if da.abs() >= TAU {
            da = -TAU;
        } else {
            while da > 0.0 {
                da -= TAU;
            }
        }

        // Split arc into max 90 degree segments.
        let ndivs = ((da.abs() / FRAC_PI_2 + 0.5) as i32).clamp(1, 5);
        let (sin, cos) = ((da / ndivs as f32) / 2.0).sin_cos();
        let kappa = (4.0 / 3.0 * (1.0 - cos) / sin).abs();
        let kappa = if let Solidity::Solid = dir {
            -kappa
        } else {
            kappa
        };

        let (mut last_point, mut last_tan) = {
            let (dy, dx) = a0.sin_cos();
            let point = Offset::new(center.x + dx * radius, center.y + dy * radius);
            let tan = Offset::new(-dy * radius * kappa, dx * radius * kappa);
            if self.is_empty() {
                self.move_to(point);
            } else {
                self.line_to(point);
            }
            (point, tan)
        };

        let inv_ndivs = (ndivs as f32).recip();

        for i in 1..ndivs + 1 {
            let a = a0 + da * (i as f32 * inv_ndivs);
            let (dy, dx) = a.sin_cos();
            let point = Offset::new(center.x + dx * radius, center.y + dy * radius);
            let tan = Offset::new(-dy * radius * kappa, dx * radius * kappa);

            self.cubic_to(last_point + last_tan, point - tan, point);

            last_point = point;
            last_tan = tan;
        }
    }
}
