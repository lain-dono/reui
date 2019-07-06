use crate::{
    cache::Winding,
    draw_api::{MOVETO, LINETO, BEZIERTO, CLOSE, WINDING},
};

pub struct Path {
    pub commands: Vec<f32>,
    pub current: [f32; 2],
}

/*
    pub fn close_path(&mut self) {
        self.append_commands(&mut [ CLOSE as f32 ]);
    }

    pub fn path_winding(&mut self, dir: Winding) {
        self.append_commands(&mut [ WINDING as f32, dir as i32 as f32 ]);
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.append_commands(&mut [ LINETO as f32, x, y ]);
    }

    pub fn bezier_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.append_commands(&mut [ BEZIERTO as f32, c1x, c1y, c2x, c2y, x, y ]);
    }
*/


impl Path {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current: [0.0, 0.0],
        }
    }

    /// Closes the last sub-path,
    /// as if a straight line had been drawn from the current point to the first point of the sub-path. 
    pub fn close(&mut self) {
        self.commands.extend_from_slice(&[ CLOSE as f32 ]);
    }

    /*
    /// Adds a new sub-path with one arc segment that consists of the arc that follows the edge of the oval bounded by the given rectangle, from startAngle radians around the oval up to startAngle + sweepAngle radians around the oval, with zero radians being the point on the right hand side of the oval that crosses the horizontal line that intersects the center of the rectangle and with positive angles going clockwise around the oval. 
    pub fn add_arc(Rect oval, double startAngle, double sweepAngle) -> void
    /// Adds a new sub-path that consists of a curve that forms the ellipse that fills the given rectangle. [...] 
    pub fn add_oval(Rect oval) -> void
    /// Adds a new sub-path that consists of the given path offset by the given offset. [...] 
    pub fn add_path(Path path, Offset offset, { Float64List matrix4 }) -> void
    /// Adds a new sub-path with a sequence of line segments that connect the given points. [...] 
    pub fn add_polygon(List<Offset> points, bool close) -> void
    /// Adds a new sub-path that consists of four lines that outline the given rectangle. 
    pub fn add_rect(Rect rect) -> void
    /// Adds a new sub-path that consists of the straight lines and curves needed to form the rounded rectangle described by the argument. 
    pub fn add_rrect(RRect rrect) -> void

    /// If the forceMoveTo argument is false, adds a straight line segment and an arc segment. [...] 
    pub fn arc_to(Rect rect, double startAngle, double sweepAngle, bool forceMoveTo) -> void
    /// Appends up to four conic curves weighted to describe an oval of radius and rotated by rotation. [...] 
    pub fn arc_to_point(Offset arcEnd, { Radius radius: Radius.zero, double rotation: 0.0, bool largeArc: false, bool clockwise: true }) -> void
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
        self.commands.extend_from_slice(&[ MOVETO as f32, x, y ]);
    }
    /// Starts a new sub-path at the given offset from the current point. 
    pub fn relative_move_to(&mut self, dx: f32, dy: f32) {
        self.move_to(self.current[0] + dx, self.current[1] + dy);
    }
    /// Adds a straight line segment from the current point to the given point. 
    pub fn line_to(&mut self, x: f32, y: f32) {
        self.commands.extend_from_slice(&[ LINETO as f32, x, y ]);
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