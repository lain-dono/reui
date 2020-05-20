use crate::math::{Corners, Offset, PartialClamp, RRect, Rect, Transform};
use std::f32::consts::PI;

// Length proportional to radius of a cubic bezier handle for 90deg arcs.
const KAPPA90: f32 = 0.552_284_8; // 0.5522847493

#[derive(Clone, Copy)]
pub enum PathCmd {
    MoveTo(Offset),
    LineTo(Offset),
    BezierTo(Offset, Offset, Offset),
    Winding(Winding),
    Close,
}

impl PathCmd {
    pub fn last_point(self) -> Option<Offset> {
        match self {
            Self::MoveTo(p) => Some(p),
            Self::LineTo(p) => Some(p),
            Self::BezierTo(_, _, p) => Some(p),
            _ => None,
        }
    }

    #[inline]
    pub fn transform(self, t: Transform) -> Self {
        match self {
            Self::MoveTo(p) => Self::MoveTo(t.apply(p)),
            Self::LineTo(p) => Self::LineTo(t.apply(p)),
            Self::BezierTo(p1, p2, p3) => Self::BezierTo(t.apply(p1), t.apply(p2), t.apply(p3)),
            Self::Winding(dir) => Self::Winding(dir),
            Self::Close => Self::Close,
        }
    }

    #[inline]
    pub fn move_to(x: f32, y: f32) -> Self {
        Self::MoveTo(Offset { x, y })
    }

    #[inline]
    pub fn line_to(x: f32, y: f32) -> Self {
        Self::LineTo(Offset { x, y })
    }

    #[inline]
    pub fn bezier_to(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> Self {
        Self::BezierTo(
            Offset { x: x1, y: y1 },
            Offset { x: x2, y: y2 },
            Offset { x: x3, y: y3 },
        )
    }

    #[inline]
    pub fn quad_to(p0: Offset, c: Offset, p1: Offset) -> Self {
        const FIX: f32 = 2.0 / 3.0;
        Self::BezierTo(p0 + (c - p0) * FIX, p1 + (c - p1) * FIX, p1)
    }

    #[inline]
    pub fn rect(Rect { min, max }: Rect) -> [Self; 5] {
        [
            Self::move_to(min.x, min.y),
            Self::line_to(min.x, max.y),
            Self::line_to(max.x, max.y),
            Self::line_to(max.x, min.y),
            Self::Close,
        ]
    }

    #[inline]
    pub fn rrect(rect: Rect, radius: Corners) -> [Self; 10] {
        let (w, h) = rect.size().into();
        let Rect { min, max } = rect;
        let halfw = w.abs() * 0.5;
        let halfh = h.abs() * 0.5;
        let sign = if w < 0.0 { -1.0 } else { 1.0 };
        let rx_bl = sign * halfw.min(radius.bl);
        let ry_bl = sign * halfh.min(radius.bl);
        let rx_br = sign * halfw.min(radius.br);
        let ry_br = sign * halfh.min(radius.br);
        let rx_tr = sign * halfw.min(radius.tr);
        let ry_tr = sign * halfh.min(radius.tr);
        let rx_tl = sign * halfw.min(radius.tl);
        let ry_tl = sign * halfh.min(radius.tl);
        let kappa = 1.0 - KAPPA90;
        [
            Self::move_to(min.x, min.y + ry_tl),
            Self::line_to(min.x, max.y - ry_bl),
            Self::bezier_to(
                min.x,
                max.y - ry_bl * kappa,
                min.x + rx_bl * kappa,
                max.y,
                min.x + rx_bl,
                max.y,
            ),
            Self::line_to(max.x - rx_br, max.y),
            Self::bezier_to(
                max.x - rx_br * kappa,
                max.y,
                max.x,
                max.y - ry_br * kappa,
                max.x,
                max.y - ry_br,
            ),
            Self::line_to(max.x, min.y + ry_tr),
            Self::bezier_to(
                max.x,
                min.y + ry_tr * kappa,
                max.x - rx_tr * kappa,
                min.y,
                max.x - rx_tr,
                min.y,
            ),
            Self::line_to(min.x + rx_tl, min.y),
            Self::bezier_to(
                min.x + rx_tl * kappa,
                min.y,
                min.x,
                min.y + ry_tl * kappa,
                min.x,
                min.y + ry_tl,
            ),
            Self::Close,
        ]
    }

    #[inline]
    pub fn ellipse(cx: f32, cy: f32, rx: f32, ry: f32) -> [Self; 6] {
        [
            Self::move_to(cx - rx, cy),
            Self::bezier_to(
                cx - rx,
                cy + ry * KAPPA90,
                cx - rx * KAPPA90,
                cy + ry,
                cx,
                cy + ry,
            ),
            Self::bezier_to(
                cx + rx * KAPPA90,
                cy + ry,
                cx + rx,
                cy + ry * KAPPA90,
                cx + rx,
                cy,
            ),
            Self::bezier_to(
                cx + rx,
                cy - ry * KAPPA90,
                cx + rx * KAPPA90,
                cy - ry,
                cx,
                cy - ry,
            ),
            Self::bezier_to(
                cx - rx * KAPPA90,
                cy - ry,
                cx - rx,
                cy - ry * KAPPA90,
                cx - rx,
                cy,
            ),
            Self::Close,
        ]
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Winding {
    CCW = 1, // Winding for solid shapes
    CW = 2,  // Winding for holes
}

pub enum PathFillType {
    NonZero = 0,
    EvenOdd = 1,
}

#[derive(Default)]
pub struct Path {
    commands: Vec<PathCmd>,
}

impl AsRef<[PathCmd]> for Path {
    fn as_ref(&self) -> &[PathCmd] {
        &self.commands
    }
}

impl Path {
    pub fn new() -> Self {
        let commands = Vec::new();
        Self { commands }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn transformed(&mut self, t: Transform) -> impl Iterator<Item = PathCmd> + '_ {
        self.commands.iter_mut().map(move |c| c.transform(t))
    }

    pub fn extend(&mut self, commands: &[PathCmd]) {
        self.commands.extend(commands);
    }

    pub fn path_winding(&mut self, dir: Winding) {
        self.commands.push(PathCmd::Winding(dir));
    }

    pub fn set_fill_type(&mut self, _fill_type: PathFillType) {
        //self.fill_type = fill_type;
        unimplemented!()
    }

    pub fn bezier_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.commands
            .push(PathCmd::bezier_to(c1x, c1y, c2x, c2y, x, y));
    }

    /// Closes the last sub-path,
    /// as if a straight line had been drawn from the current point to the first point of the sub-path.
    pub fn close(&mut self) {
        self.commands.push(PathCmd::Close);
    }

    /*
    /// Adds a new sub-path with one arc segment that consists of the arc that follows the edge of the oval
    /// bounded by the given rectangle,
    /// from startAngle radians around the oval up to startAngle + sweepAngle radians around the oval,
    /// with zero radians being the point on the right hand side of the oval that crosses the horizontal line
    /// that intersects the center of the rectangle and with positive angles going clockwise around the oval.
    pub fn add_arc(Rect oval, double startAngle, double sweepAngle) -> void
    */

    /// Adds a new sub-path that consists of a curve that forms the ellipse that fills the given rectangle. [...]
    pub fn add_oval(&mut self, oval: Rect) {
        let [cx, cy] = [oval.min.x, oval.min.y];
        //let Offset { x: cx, y: cy } = oval.center();
        let Offset { x: rx, y: ry } = oval.size();

        self.commands.extend(&PathCmd::ellipse(cx, cy, rx, ry));
    }

    pub fn add_ellipse(&mut self, center: Offset, radius: Offset) {
        self.commands
            .extend(&PathCmd::ellipse(center.x, center.y, radius.x, radius.y));
    }

    pub fn add_circle(&mut self, c: Offset, r: f32) {
        self.commands.extend(&PathCmd::ellipse(c.x, c.y, r, r));
    }

    /*
    /// Adds a new sub-path that consists of the given path offset by the given offset. [...]
    pub fn add_path(Path path, Offset offset, { Float64List matrix4 }) -> void
    /// Adds a new sub-path with a sequence of line segments that connect the given points. [...]
    pub fn add_polygon(List<Offset> points, bool close) -> void
    */

    /// Adds a new sub-path that consists of four lines that outline the given rectangle.
    pub fn add_rect(&mut self, rect: Rect) {
        self.commands.extend(&PathCmd::rect(rect));
    }

    /// Adds a new sub-path that consists of the straight lines and curves needed to form the rounded rectangle described by the argument.
    pub fn add_rrect(&mut self, RRect { rect, radius }: RRect) {
        if radius.tl < 0.1 && radius.tr < 0.1 && radius.br < 0.1 && radius.bl < 0.1 {
            self.add_rect(rect);
        } else {
            self.commands.extend(&PathCmd::rrect(rect, radius));
        }
    }

    /*
    /// If the forceMoveTo argument is false, adds a straight line segment and an arc segment. [...]
    pub fn arc_to(Rect rect, double startAngle, double sweepAngle, bool forceMoveTo) -> void
    /// Appends up to four conic curves weighted to describe an oval of radius and rotated by rotation. [...]
    pub fn arc_to_point(Offset arcEnd, {
        Radius radius: Radius.zero, double rotation: 0.0, bool largeArc: false, bool clockwise: true }) -> void
    /// Creates a PathMetrics object for this path. [...]
    pub fn compute_metrics({bool forceClosed: false }) -> PathMetrics
    /// Adds a bezier segment that curves from the current point to the given point (x2,y2),
    /// using the control points (x1,y1) and the weight w.
    /// If the weight is greater than 1, then the curve is a hyperbola;
    /// if the weight equals 1, it's a parabola; and if it is less than 1, it is an ellipse.
    pub fn conic_to(double x1, double y1, double x2, double y2, double w) -> void
    /// Tests to see if the given point is within the path. (That is, whether the point would be in the visible portion of the path if the path was used with Canvas.clipPath.) [...]
    pub fn contains(Offset point) -> bool
    /// Adds a cubic bezier segment that curves from the current point to the given point (x3,y3), using the control points (x1,y1) and (x2,y2).
    pub fn cubic_to(double x1, double y1, double x2, double y2, double x3, double y3) -> void
    /// Adds the given path to this path by extending the current segment of this path with the the first segment of the given path. [...]
    pub fn extend_with_path(Path path, Offset offset, { Float64List matrix4 }) -> void
    /// Computes the bounding rectangle for this path. [...]
    pub fn bounds() -> Rect
    */

    /// Starts a new sub-path at the given coordinate.
    pub fn move_to(&mut self, p: Offset) {
        self.commands.push(PathCmd::MoveTo(p));
    }
    /// Adds a straight line segment from the current point to the given point.
    pub fn line_to(&mut self, p: Offset) {
        self.commands.push(PathCmd::LineTo(p));
    }

    /*
    /// Adds a quadratic bezier segment that curves from the current point to the given point (x2,y2), using the control point (x1,y1).
    pub fn quadratic_bezier_to(double x1, double y1, double x2, double y2) -> void
    /// Appends up to four conic curves weighted to describe an oval of radius and rotated by rotation. [...]
    pub fn relative_arc_to_point(Offset arcEndDelta, { Radius radius: Radius.zero, double rotation: 0.0, bool largeArc: false, bool clockwise: true }) -> void
    /// Adds a bezier segment that curves from the current point to the point at the offset (x2,y2) from the current point, using the control point at the offset (x1,y1) from the current point and the weight w. If the weight is greater than 1, then the curve is a hyperbola; if the weight equals 1, it's a parabola; and if it is less than 1, it is an ellipse.
    pub fn relative_conic_to(double x1, double y1, double x2, double y2, double w) -> void
    /// Adds a cubic bezier segment that curves from the current point to the point at the offset (x3,y3) from the current point, using the control points at the offsets (x1,y1) and (x2,y2) from the current point.
    pub fn relative_cubic_to(double x1, double y1, double x2, double y2, double x3, double y3) -> void
    /// Adds a quadratic bezier segment that curves from the current point to the point at the offset (x2,y2) from the current point, using the control point at the offset (x1,y1) from the current point.
    pub fn relative_quadratic_bezier_to(double x1, double y1, double x2, double y2) -> void
    /// Clears the Path object of all sub-paths, returning it to the same state it had when it was created.
    /// The current point is reset to the origin.
    pub fn reset() -> void
    */

    /*
    /// Returns a copy of the path with all the segments of every sub-path translated by the given offset.
    pub fn shift(Offset offset) -> Path
    /// Returns a copy of the path with all the segments of every sub-path transformed by the given matrix.
    pub fn transform(Float64List matrix4) -> Path
    */
}

impl Path {
    pub fn arc(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, dir: Winding) {
        // Clamp angles
        let mut da = a1 - a0;
        if dir == Winding::CW {
            if da.abs() >= PI * 2.0 {
                da = PI * 2.0;
            } else {
                while da < 0.0 {
                    da += PI * 2.0;
                }
            }
        } else if da.abs() >= PI * 2.0 {
            da = -PI * 2.0;
        } else {
            while da > 0.0 {
                da -= PI * 2.0;
            }
        }

        // Split arc into max 90 degree segments.
        let ndivs = ((da.abs() / (PI * 0.5) + 0.5) as i32).clamp(1, 5);
        let hda = (da / ndivs as f32) / 2.0;
        let kappa = (4.0 / 3.0 * (1.0 - hda.cos()) / hda.sin()).abs();
        let kappa = if dir == Winding::CCW { -kappa } else { kappa };

        let mut vals = [PathCmd::Close; 1 + 5];
        let mut nvals = 0;
        let (mut px, mut py, mut ptanx, mut ptany) = (0.0, 0.0, 0.0, 0.0);
        for i in 0..=ndivs {
            let a = a0 + da * (i as f32 / ndivs as f32);
            let dx = a.cos();
            let dy = a.sin();
            let x = cx + dx * r;
            let y = cy + dy * r;
            let tanx = -dy * r * kappa;
            let tany = dx * r * kappa;

            if i == 0 {
                vals[nvals] = if !self.commands.is_empty() {
                    PathCmd::line_to(x, y)
                } else {
                    PathCmd::move_to(x, y)
                };
                nvals += 1;
            } else {
                vals[nvals] = PathCmd::bezier_to(px + ptanx, py + ptany, x - tanx, y - tany, x, y);
                nvals += 1;
            }
            px = x;
            py = y;
            ptanx = tanx;
            ptany = tany;
        }

        self.commands.extend(&vals[..nvals]);
    }
}
