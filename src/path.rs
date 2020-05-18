use crate::{
    canvas::Winding,
    math::{Offset, PartialClamp, RRect, Rect},
    recorder::{BEZIERTO, CLOSE, LINETO, MOVETO, WINDING},
};
use smallvec::{Array, SmallVec};
use std::f32::consts::PI;

// Length proportional to radius of a cubic bezier handle for 90deg arcs.
const KAPPA90: f32 = 0.552_284_8; // 0.5522847493

pub enum PathFillType {
    NonZero = 0,
    EvenOdd = 1,
}

#[derive(Default)]
pub struct Path<A: Array<Item = f32>> {
    current: [f32; 2],
    commands: SmallVec<A>,
}

impl<A: Array<Item = f32>> std::ops::Deref for Path<A> {
    type Target = [f32];
    fn deref(&self) -> &[f32] {
        &self.commands[..]
    }
}

impl<A: Array<Item = f32>> std::ops::DerefMut for Path<A> {
    fn deref_mut(&mut self) -> &mut [f32] {
        &mut self.commands[..]
    }
}

impl<A: Array<Item = f32>> Path<A> {
    pub fn new() -> Self {
        Self {
            commands: SmallVec::new(),
            current: [0.0, 0.0],
        }
    }

    pub fn clear(&mut self) {
        self.commands.clear();
        self.current = [0.0, 0.0];
    }

    pub fn path_winding(&mut self, dir: Winding) {
        self.commands
            .extend_from_slice(&[WINDING as f32, dir as i32 as f32]);
    }

    pub fn set_fill_type(&mut self, _fill_type: PathFillType) {
        //self.fill_type = fill_type;
        unimplemented!()
    }

    pub fn bezier_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.commands
            .extend_from_slice(&[BEZIERTO as f32, c1x, c1y, c2x, c2y, x, y]);
    }

    /// Closes the last sub-path,
    /// as if a straight line had been drawn from the current point to the first point of the sub-path.
    pub fn close(&mut self) {
        self.commands.extend_from_slice(&[CLOSE as f32]);
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
        self.commands.extend_from_slice(&[
            MOVETO as f32,
            cx - rx,
            cy,
            BEZIERTO as f32,
            cx - rx,
            cy + ry * KAPPA90,
            cx - rx * KAPPA90,
            cy + ry,
            cx,
            cy + ry,
            BEZIERTO as f32,
            cx + rx * KAPPA90,
            cy + ry,
            cx + rx,
            cy + ry * KAPPA90,
            cx + rx,
            cy,
            BEZIERTO as f32,
            cx + rx,
            cy - ry * KAPPA90,
            cx + rx * KAPPA90,
            cy - ry,
            cx,
            cy - ry,
            BEZIERTO as f32,
            cx - rx * KAPPA90,
            cy - ry,
            cx - rx,
            cy - ry * KAPPA90,
            cx - rx,
            cy,
            CLOSE as f32,
        ]);
    }

    pub fn add_circle(&mut self, c: Offset, r: f32) {
        self.add_oval(Rect::from_ltwh(c.x, c.y, r, r));
    }

    /*
    /// Adds a new sub-path that consists of the given path offset by the given offset. [...]
    pub fn add_path(Path path, Offset offset, { Float64List matrix4 }) -> void
    /// Adds a new sub-path with a sequence of line segments that connect the given points. [...]
    pub fn add_polygon(List<Offset> points, bool close) -> void
    */

    /// Adds a new sub-path that consists of four lines that outline the given rectangle.
    pub fn add_rect(&mut self, rect: Rect) {
        self.commands.extend_from_slice(&[
            MOVETO as f32,
            rect.min.x,
            rect.min.y,
            LINETO as f32,
            rect.min.x,
            rect.max.y,
            LINETO as f32,
            rect.max.x,
            rect.max.y,
            LINETO as f32,
            rect.max.x,
            rect.min.y,
            CLOSE as f32,
        ]);
    }
    /// Adds a new sub-path that consists of the straight lines and curves needed to form the rounded rectangle described by the argument.
    pub fn add_rrect(&mut self, rr: RRect) {
        let (x, y, w, h) = (rr.rect.min.x, rr.rect.min.y, rr.width(), rr.height());
        if rr.radius.tl < 0.1 && rr.radius.tr < 0.1 && rr.radius.br < 0.1 && rr.radius.bl < 0.1 {
            self.add_rect(rr.rect);
        } else {
            let halfw = w.abs() * 0.5;
            let halfh = h.abs() * 0.5;
            let sign = if w < 0.0 { -1.0 } else { 1.0 };
            let rx_bl = sign * halfw.min(rr.radius.bl);
            let ry_bl = sign * halfh.min(rr.radius.bl);
            let rx_br = sign * halfw.min(rr.radius.br);
            let ry_br = sign * halfh.min(rr.radius.br);
            let rx_tr = sign * halfw.min(rr.radius.tr);
            let ry_tr = sign * halfh.min(rr.radius.tr);
            let rx_tl = sign * halfw.min(rr.radius.tl);
            let ry_tl = sign * halfh.min(rr.radius.tl);
            let kappa = 1.0 - KAPPA90;
            self.commands.extend_from_slice(&[
                MOVETO as f32,
                x,
                y + ry_tl,
                LINETO as f32,
                x,
                y + h - ry_bl,
                BEZIERTO as f32,
                x,
                y + h - ry_bl * kappa,
                x + rx_bl * kappa,
                y + h,
                x + rx_bl,
                y + h,
                LINETO as f32,
                x + w - rx_br,
                y + h,
                BEZIERTO as f32,
                x + w - rx_br * kappa,
                y + h,
                x + w,
                y + h - ry_br * kappa,
                x + w,
                y + h - ry_br,
                LINETO as f32,
                x + w,
                y + ry_tr,
                BEZIERTO as f32,
                x + w,
                y + ry_tr * kappa,
                x + w - rx_tr * kappa,
                y,
                x + w - rx_tr,
                y,
                LINETO as f32,
                x + rx_tl,
                y,
                BEZIERTO as f32,
                x + rx_tl * kappa,
                y,
                x,
                y + ry_tl * kappa,
                x,
                y + ry_tl,
                CLOSE as f32,
            ]);
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
    pub fn move_to(&mut self, x: f32, y: f32) {
        self.commands.extend_from_slice(&[MOVETO as f32, x, y]);
    }
    /// Starts a new sub-path at the given offset from the current point.
    pub fn relative_move_to(&mut self, dx: f32, dy: f32) {
        self.move_to(self.current[0] + dx, self.current[1] + dy);
    }
    /// Adds a straight line segment from the current point to the given point.
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.commands.extend_from_slice(&[LINETO as f32, x, y]);
    }
    /// Adds a straight line segment from the current point to the point at the given offset from the current point.
    pub fn relative_line_to(&mut self, dx: f32, dy: f32) {
        self.line_to(self.current[0] + dx, self.current[1] + dy);
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

impl<A: Array<Item = f32>> Path<A> {
    pub fn arc(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, dir: Winding) {
        let mov = if !self.commands.is_empty() {
            LINETO
        } else {
            MOVETO
        };

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

        let mut vals = [0f32; 3 + 5 * 7];
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
                vals[nvals] = mov as f32;
                vals[nvals + 1] = x;
                vals[nvals + 2] = y;
                nvals += 3;
            } else {
                vals[nvals] = BEZIERTO as f32;
                vals[nvals + 1] = px + ptanx;
                vals[nvals + 2] = py + ptany;
                vals[nvals + 3] = x - tanx;
                vals[nvals + 4] = y - tany;
                vals[nvals + 5] = x;
                vals[nvals + 6] = y;
                nvals += 7;
            }
            px = x;
            py = y;
            ptanx = tanx;
            ptany = tany;
        }

        self.commands.extend_from_slice(&vals[..nvals]);
    }
}
