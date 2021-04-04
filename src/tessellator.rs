use crate::{
    geom::{Offset, Rect},
    paint::{LineCap, LineJoin},
    path::{Command, FillRule, Solidity},
    pipeline::{Batch, Vertex},
};
use std::{
    cmp::Ordering,
    f32::consts::{PI, TAU},
    ops::Range,
};

mod util {
    use crate::geom::Offset;

    // Adapted from libcollections/vec.rs in Rust
    // Primary author in Rust: Michael Darakananda
    pub fn retain_mut<T>(vec: &mut Vec<T>, mut f: impl FnMut(&mut T) -> bool) {
        let len = vec.len();
        let mut del = 0;
        {
            let v = &mut **vec;

            for i in 0..len {
                if !f(&mut v[i]) {
                    del += 1;
                } else if del > 0 {
                    v.swap(i - del, i);
                }
            }
        }
        if del > 0 {
            vec.truncate(len - del);
        }
    }

    #[inline]
    pub fn approx_eq(a: Offset, b: Offset, epsilon: f32) -> bool {
        let x = (a.x - b.x).abs() < epsilon;
        let y = (a.y - b.y).abs() < epsilon;
        x && y
    }
}

const DEFAULT_BOUNDS: Rect = Rect {
    min: Offset::new(1e6, 1e6),
    max: Offset::new(-1e6, -1e6),
};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Convexity {
    Unknown,
    Concave,
    Convex,
}

impl Default for Convexity {
    fn default() -> Self {
        Self::Unknown
    }
}

bitflags::bitflags!(
    #[derive(Default)]
    pub struct PointFlags: u32 {
        const CORNER = 0x01;
        const LEFT = 0x02;
        const BEVEL = 0x04;
        const INNERBEVEL = 0x08;
    }
);

#[derive(Clone, Default)]
struct Point {
    pos: Offset, // position
    dir: Offset, // direction
    ext: Offset, // extrusions
    len: f32,
    flags: PointFlags,
}

impl Point {
    pub fn new(pos: Offset, flags: PointFlags) -> Self {
        Self {
            pos,
            flags,
            ..Self::default()
        }
    }

    #[inline]
    pub fn polygon_area(points: &[Self]) -> f32 {
        fn triarea2(a: Offset, b: Offset, c: Offset) -> f32 {
            let (ba, ca) = (b - a, c - a);
            ca.x * ba.y - ba.x * ca.y
        }

        0.5 * points
            .windows(3)
            .map(|w| triarea2(w[0].pos, w[1].pos, w[2].pos))
            .sum::<f32>()
    }

    #[inline]
    fn _polygon_area(pts: &[Self]) -> f32 {
        let mut area = 0.0;
        let a = &pts[0];
        for i in 2..pts.len() {
            let b = &pts[i - 1];
            let c = &pts[i];
            area += (c.pos - a.pos).cross(b.pos - a.pos);
        }
        area * 0.5
    }

    #[inline]
    pub fn set_direction(&mut self, mut dir: Offset) {
        let xx = dir.x * dir.x;
        let yy = dir.y * dir.y;
        let d = (xx + yy).sqrt();
        if d > 1e-6 {
            let id = d.recip();
            dir.x *= id;
            dir.y *= id;
        }
        self.dir = dir;
        self.len = d;
    }
}

pub enum Draw {
    Convex {
        base_vertex: i32,
        indices: Range<u32>,
    },
    Concave {
        base_vertex: i32,
        fill: Range<u32>,
        stroke: Range<u32>,
        quad: Range<u32>,
    },
}

pub struct Contour {
    point_range: Range<usize>,
    bevel: usize,
    closed: bool,
    solidity: Solidity,
    convexity: Convexity,
}

impl Default for Contour {
    fn default() -> Self {
        Self {
            point_range: 0..0,
            closed: false,
            bevel: 0,
            solidity: Solidity::default(),
            convexity: Convexity::default(),
        }
    }
}

impl Contour {
    fn point_pairs<'a>(&self, points: &'a [Point]) -> PointPairsIter<'a> {
        PointPairsIter {
            points: &points[self.point_range.clone()],
            current: 0,
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.point_range.end.saturating_sub(self.point_range.start)
    }
}

struct PointPairsIter<'a> {
    points: &'a [Point],
    current: usize,
}

impl<'a> Iterator for PointPairsIter<'a> {
    type Item = (&'a Point, &'a Point);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.points.get(self.current);

        let prev = if self.current == 0 {
            self.points.last()
        } else {
            self.points.get(self.current - 1)
        };

        self.current += 1;

        current.and_then(|some_curr| prev.map(|some_prev| (some_prev, some_curr)))
    }
}

pub struct Tessellator {
    points: Vec<Point>,
    contours: Vec<Contour>,
    bounds: Rect,
}

impl Default for Tessellator {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            contours: Vec::new(),
            bounds: DEFAULT_BOUNDS,
        }
    }
}

impl Tessellator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bounds(&self) -> Rect {
        self.bounds
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.contours.clear();
        self.bounds = DEFAULT_BOUNDS;
    }

    fn is_convex(&self) -> bool {
        self.contours.len() == 1 && self.contours[0].convexity == Convexity::Convex
    }

    pub fn contains(&self, p2: Offset, fill_rule: FillRule) -> bool {
        // Early out if point is outside the bounding rectangle
        if !self.bounds.contains(p2) {
            return false;
        }

        fn extract_pos(pair: (&Point, &Point)) -> (Offset, Offset) {
            (pair.0.pos, pair.1.pos)
        }

        fn is_left(p0: Offset, p1: Offset, p2: Offset) -> f32 {
            (p1.x - p0.x) * (p2.y - p0.y) - (p1.y - p0.y) * (p2.x - p0.x)
        }

        if fill_rule == FillRule::EvenOdd {
            self.contours.iter().any(|contour| {
                let mut crossing = false;

                for (p0, p1) in contour.point_pairs(&self.points).map(extract_pos) {
                    if (p1.y > p2.y) != (p0.y > p2.y)
                        && (p2.x < (p0.x - p1.x) * (p2.y - p1.y) / (p0.y - p1.y) + p1.x)
                    {
                        crossing = !crossing;
                    }
                }
                crossing
            })
        } else {
            // NonZero
            self.contours.iter().any(|contour| {
                let mut winding_number: i32 = 0;

                for (p0, p1) in contour.point_pairs(&self.points).map(extract_pos) {
                    if p0.y <= p2.y {
                        if p1.y > p2.y && is_left(p0, p1, p2) > 0.0 {
                            winding_number = winding_number.wrapping_add(1);
                        }
                    } else if p1.y <= p2.y && is_left(p0, p1, p2) < 0.0 {
                        winding_number = winding_number.wrapping_sub(1);
                    }
                }

                winding_number != 0
            })
        }
    }

    /// # Panics
    pub fn flatten(
        &mut self,
        commands: impl Iterator<Item = Command>,
        tess_tol: f32,
        dist_tol: f32,
    ) {
        // clear all
        self.clear();

        // Convert path commands to a set of contours
        for cmd in commands {
            match cmd {
                Command::MoveTo(p) => {
                    self.add_contour();
                    self.add_point(p, dist_tol, PointFlags::CORNER);
                }
                Command::LineTo(p) => {
                    self.add_point(p, dist_tol, PointFlags::CORNER);
                }
                Command::BezierTo(p1, p2, p3) => {
                    if let Some(p0) = self.points.last().cloned() {
                        self.tesselate_bezier([p0.pos, p1, p2, p3], tess_tol, dist_tol);
                    }
                }
                Command::Solid => {
                    if let Some(contour) = self.contours.last_mut() {
                        contour.solidity = Solidity::Solid;
                    }
                }
                Command::Hole => {
                    if let Some(contour) = self.contours.last_mut() {
                        contour.solidity = Solidity::Hole;
                    }
                }
                Command::Close => {
                    if let Some(contour) = self.contours.last_mut() {
                        contour.closed = true;
                    }
                }
            }
        }

        let all_points = &mut self.points;
        let bounds = &mut self.bounds;

        util::retain_mut(&mut self.contours, |contour| {
            let mut points = &mut all_points[contour.point_range.clone()];

            // If the first and last points are the same, remove the last, mark as closed contour.
            if let (Some(p0), Some(p1)) = (points.last(), points.first()) {
                if util::approx_eq(p0.pos, p1.pos, dist_tol) {
                    contour.point_range.end -= 1;
                    contour.closed = true;
                    points = &mut all_points[contour.point_range.clone()];
                }
            }

            if points.len() < 2 {
                return false;
            }

            // Enforce solidity by reversing the winding.
            let area = Point::polygon_area(points);
            if contour.solidity == Solidity::Solid && area < 0.0 {
                points.reverse();
            }
            if contour.solidity == Solidity::Hole && area > 0.0 {
                points.reverse();
            }

            for i in 0..contour.len() {
                let p1 = points.get(i).cloned().unwrap();

                let p0 = if i == 0 {
                    points.last_mut().unwrap()
                } else {
                    points.get_mut(i - 1).unwrap()
                };

                p0.set_direction(p1.pos - p0.pos);

                bounds.min.x = bounds.min.x.min(p0.pos.x);
                bounds.min.y = bounds.min.y.min(p0.pos.y);
                bounds.max.x = bounds.max.x.max(p0.pos.x);
                bounds.max.y = bounds.max.y.max(p0.pos.y);
            }

            true
        });
    }

    pub(crate) fn expand_fill(
        &mut self,
        batch: &mut Batch,
        fringe_width: f32,
        line_join: LineJoin,
        miter_limit: f32,
    ) -> Draw {
        self.calculate_joins(fringe_width, line_join, miter_limit);

        let has_fringe = fringe_width > 0.0;

        let convex = self.is_convex();

        let base_vertex = batch.base_vertex();

        // TODO: woff = 0.0 produces no artifaacts for small sizes
        let woff = 0.5 * fringe_width;
        //let woff = 0.0; // Makes everything thicker

        let fill;
        let stroke;

        if has_fringe {
            let base_index = batch.base_index();
            let mut offset = 0;

            for contour in &self.contours {
                let start = batch.base_vertex();
                for (p0, p1) in contour.point_pairs(&self.points) {
                    let uv = [0.5, 1.0];
                    if p1.flags.contains(PointFlags::BEVEL) {
                        if p1.flags.contains(PointFlags::LEFT) {
                            batch.push(Vertex::new(p1.pos + p1.ext * woff, uv));
                        } else {
                            let p0 = p1.pos + Offset::new(p0.dir.y, -p0.dir.x) * woff;
                            let p1 = p1.pos + Offset::new(p1.dir.y, -p1.dir.x) * woff;
                            batch.push(Vertex::new(p0, uv));
                            batch.push(Vertex::new(p1, uv));
                        }
                    } else {
                        batch.push(Vertex::new(p1.pos + p1.ext * woff, uv));
                    }
                }

                let num_batch = batch.base_vertex() - start;
                batch.fan(offset, num_batch);
                offset += num_batch as u32;
            }

            fill = base_index..batch.base_index();
            let base_index = batch.base_index();

            for contour in &self.contours {
                let (rw, ru) = (fringe_width - woff, 1.0);
                let (lw, lu) = if convex {
                    // Create only half a fringe for convex shapes so that
                    // the shape can be rendered without stenciling.
                    (woff, 0.5)
                } else {
                    (fringe_width + woff, 0.0)
                };

                let start = batch.base_vertex();

                for (p0, p1) in contour.point_pairs(&self.points) {
                    if p1
                        .flags
                        .contains(PointFlags::BEVEL | PointFlags::INNERBEVEL)
                    {
                        bevel_join(batch, p0, &p1, [lw, rw, lu, ru]);
                    } else {
                        batch.push(Vertex::new(p1.pos + (p1.ext * lw), [lu, 1.0]));
                        batch.push(Vertex::new(p1.pos - (p1.ext * rw), [ru, 1.0]));
                    }
                }

                // Loop it
                let p0 = batch[start].pos;
                let p1 = batch[start + 1].pos;
                batch.push(Vertex::new(p0, [lu, 1.0]));
                batch.push(Vertex::new(p1, [ru, 1.0]));

                let num_batch = batch.base_vertex() - start;
                batch.strip(offset, num_batch);
                offset += num_batch as u32;
            }

            stroke = base_index..batch.base_index();
        } else {
            let base_index = batch.base_index();
            let mut offset = 0;
            for contour in &mut self.contours {
                let points = &self.points[contour.point_range.clone()];

                let start = batch.base_vertex();
                for point in points {
                    batch.push(Vertex::new(point.pos, [0.5, 1.0]));
                }

                let num_batch = batch.base_vertex() - start;
                batch.fan(offset, num_batch);
                offset += num_batch as u32;
            }
            fill = base_index..batch.base_index();
            stroke = batch.base_index()..batch.base_index();
        }

        if convex {
            Draw::Convex {
                base_vertex,
                indices: fill.start..stroke.end,
            }
        } else {
            let Rect { min, max } = self.bounds;
            let quad = batch.push_strip(
                (batch.base_vertex() - base_vertex) as u32,
                &[
                    Vertex::new([max.x, max.y], [0.5, 1.0]),
                    Vertex::new([max.x, min.y], [0.5, 1.0]),
                    Vertex::new([min.x, max.y], [0.5, 1.0]),
                    Vertex::new([min.x, min.y], [0.5, 1.0]),
                ],
            );
            Draw::Concave {
                base_vertex,
                fill,
                stroke,
                quad,
            }
        }
    }

    pub(crate) fn expand_stroke(
        &mut self,
        batch: &mut Batch,
        stroke_width: f32,
        fringe_width: f32,
        cap_start: LineCap,
        cap_end: LineCap,
        join: LineJoin,
        miter_limit: f32,
        tess_tol: f32,
    ) -> Range<u32> {
        let ncap = curve_divisions(stroke_width, PI, tess_tol);
        let stroke_width = stroke_width + (fringe_width * 0.5);
        self.calculate_joins(stroke_width, join, miter_limit);

        // Disable the gradient used for antialiasing when antialiasing is not enabled.
        let (u0, u1) = if fringe_width == 0.0 {
            (0.5, 0.5)
        } else {
            (0.0, 1.0)
        };

        let base_index = batch.base_index();
        let mut offset = 0;

        for contour in &self.contours {
            let start = batch.base_vertex();
            for (i, (p0, p1)) in contour.point_pairs(&self.points).enumerate() {
                // Add start cap
                if !contour.closed && i == 1 {
                    match cap_start {
                        LineCap::Butt => butt_cap_start(
                            batch,
                            p0,
                            p0,
                            stroke_width,
                            -fringe_width * 0.5,
                            fringe_width,
                            (u0, u1),
                        ),
                        LineCap::Square => butt_cap_start(
                            batch,
                            p0,
                            p0,
                            stroke_width,
                            stroke_width - fringe_width,
                            fringe_width,
                            (u0, u1),
                        ),
                        LineCap::Round => {
                            round_cap_start(batch, p0, p0, stroke_width, ncap as usize, (u0, u1))
                        }
                    }
                }

                if (i > 0 && i < contour.len() - 1) || contour.closed {
                    if p1.flags.contains(PointFlags::BEVEL)
                        || p1.flags.contains(PointFlags::INNERBEVEL)
                    {
                        let args = [stroke_width, stroke_width, u0, u1];
                        if join == LineJoin::Round {
                            round_join(batch, p0, p1, args, ncap as usize);
                        } else {
                            bevel_join(batch, p0, p1, args);
                        }
                    } else {
                        batch.push(Vertex::new(p1.pos + (p1.ext * stroke_width), [u0, 1.0]));
                        batch.push(Vertex::new(p1.pos - (p1.ext * stroke_width), [u1, 1.0]));
                    }
                }

                // Add end cap
                if !contour.closed && i == contour.len() - 1 {
                    match cap_end {
                        LineCap::Butt => butt_cap_end(
                            batch,
                            p1,
                            p0,
                            stroke_width,
                            -fringe_width * 0.5,
                            fringe_width,
                            (u0, u1),
                        ),
                        LineCap::Square => butt_cap_end(
                            batch,
                            &p1,
                            &p0,
                            stroke_width,
                            stroke_width - fringe_width,
                            fringe_width,
                            (u0, u1),
                        ),
                        LineCap::Round => {
                            round_cap_end(batch, &p1, &p0, stroke_width, ncap as usize, (u0, u1))
                        }
                    }
                }
            }

            if contour.closed {
                let p0 = batch[start].pos;
                let p1 = batch[start + 1].pos;
                batch.push(Vertex::new(p0, [u0, 1.0]));
                batch.push(Vertex::new(p1, [u1, 1.0]));
            }

            let num_batch = batch.base_vertex() - start;
            batch.strip(offset, num_batch);
            offset += num_batch as u32;
        }

        base_index..batch.base_index()
    }

    fn add_point(&mut self, point: Offset, dist_tol: f32, flags: PointFlags) {
        if let Some(contour) = self.contours.last_mut() {
            // If last point equals this new point just OR the flags and ignore the new point
            if let Some(last_point) = self.points.get_mut(contour.point_range.end) {
                if util::approx_eq(last_point.pos, point, dist_tol) {
                    last_point.flags |= flags;
                    return;
                }
            }

            self.points.push(Point::new(point, flags));
            contour.point_range.end += 1;
        }
    }

    fn add_contour(&mut self) {
        self.contours.push(Contour {
            point_range: self.points.len()..self.points.len(),
            ..Contour::default()
        });
    }

    fn calculate_joins(&mut self, width: f32, line_join: LineJoin, miter_limit: f32) {
        let inv_width = if width > 0.0 { 1.0 / width } else { 0.0 };

        for contour in &mut self.contours {
            let points = &mut self.points[contour.point_range.clone()];
            let mut nleft = 0;

            contour.bevel = 0;

            let mut x_sign = 0;
            let mut y_sign = 0;
            let mut x_first_sign = 0; // Sign of first nonzero edge vector x
            let mut y_first_sign = 0; // Sign of first nonzero edge vector y
            let mut x_flips = 0; // Number of sign changes in x
            let mut y_flips = 0; // Number of sign changes in y

            for i in 0..points.len() {
                let p0 = if i == 0 {
                    points.get(points.len() - 1).cloned().unwrap()
                } else {
                    points.get(i - 1).cloned().unwrap()
                };

                let p1 = points.get_mut(i).unwrap();

                let dl0 = Offset::new(p0.dir.y, -p0.dir.x);
                let dl1 = Offset::new(p1.dir.y, -p1.dir.x);

                // Calculate extrusions
                p1.ext = (dl0 + dl1) * 0.5;

                let dmr2 = p1.ext.x * p1.ext.x + p1.ext.y * p1.ext.y;
                if dmr2 > 0.000_001 {
                    p1.ext *= (1.0 / dmr2).min(600.0);
                }

                // Clear flags, but keep the corner.
                p1.flags &= PointFlags::CORNER;

                // Keep track of left turns.
                let cross = p1.dir.x * p0.dir.y - p0.dir.x * p1.dir.y;
                if cross > 0.0 {
                    nleft += 1;
                    p1.flags |= PointFlags::LEFT;
                }

                // Determine sign for convexity
                match p1.dir.x.partial_cmp(&0.0) {
                    Some(Ordering::Greater) => {
                        match x_sign.cmp(&0) {
                            Ordering::Less => x_flips += 1,
                            Ordering::Equal => x_first_sign = 1,
                            Ordering::Greater => (),
                        }
                        x_sign = 1;
                    }
                    Some(Ordering::Less) => {
                        match x_sign.cmp(&0) {
                            Ordering::Less => (),
                            Ordering::Equal => x_first_sign = -1,
                            Ordering::Greater => x_flips += 1,
                        }
                        x_sign = -1;
                    }
                    _ => (),
                }

                match p1.dir.y.partial_cmp(&0.0) {
                    Some(Ordering::Greater) => {
                        match y_sign.cmp(&0) {
                            Ordering::Less => y_flips += 1,
                            Ordering::Equal => y_first_sign = 1,
                            Ordering::Greater => (),
                        }
                        y_sign = 1;
                    }
                    Some(Ordering::Less) => {
                        match y_sign.cmp(&0) {
                            Ordering::Less => (),
                            Ordering::Equal => y_first_sign = -1,
                            Ordering::Greater => y_flips += 1,
                        }
                        y_sign = -1;
                    }
                    _ => (),
                }

                // Calculate if we should use bevel or miter for inner join.
                let limit = (p0.len.min(p1.len) * inv_width).max(1.01);

                if (dmr2 * limit * limit) < 1.0 {
                    p1.flags |= PointFlags::INNERBEVEL;
                }

                // Check to see if the corner needs to be beveled.
                if p1.flags.contains(PointFlags::CORNER)
                    && ((dmr2 * miter_limit * miter_limit) < 1.0
                        || line_join == LineJoin::Bevel
                        || line_join == LineJoin::Round)
                {
                    p1.flags |= PointFlags::BEVEL;
                }

                contour.bevel += p1
                    .flags
                    .contains(PointFlags::BEVEL | PointFlags::INNERBEVEL)
                    as usize;
            }

            x_flips += (x_sign != 0 && x_first_sign != 0 && x_sign != x_first_sign) as i32;
            y_flips += (y_sign != 0 && y_first_sign != 0 && y_sign != y_first_sign) as i32;

            let convex = x_flips == 2 && y_flips == 2;

            contour.convexity = if nleft == points.len() && convex {
                Convexity::Convex
            } else {
                Convexity::Concave
            };
        }
    }

    // Adaptive forward differencing for bezier tesselation.
    // See Lien, Sheue-Ling, Michael Shantz, and Vaughan Pratt.
    // "Adaptive forward differencing for rendering curves and surfaces."
    // ACM SIGGRAPH Computer Graphics. Vol. 21. No. 4. ACM, 1987.
    fn tesselate_bezier(&mut self, [p0, p1, p2, p3]: [Offset; 4], tess_tol: f32, dist_tol: f32) {
        const AFD_ONE: i32 = 1 << 10;

        // Power basis.
        let a = p1 * 3.0 - p0 - p2 * 3.0 + p3;
        let b = p0 * 3.0 - p1 * 6.0 + p2 * 3.0;
        let c = p1 * 3.0 - p0 * 3.0;

        // Transform to forward difference basis (stepsize 1)
        let mut d0 = p0;
        let mut d1 = a + b + c;
        let mut d2 = a * 6.0 + b * 2.0;
        let mut d3 = a * 6.0;
        let mut times: i32 = 0;
        let mut stepsize: i32 = AFD_ONE;
        let tol = tess_tol * 4.0;
        while times < AFD_ONE {
            // Flatness measure.
            let mut flatness = d2.magnitude_sq() + d3.magnitude_sq();

            // Go to higher resolution if we're moving a lot
            // or overshooting the end.
            while flatness > tol && stepsize > 1 || times + stepsize > AFD_ONE {
                // Apply L to the curve. Increase curve resolution.
                d1 = d1 * 0.50 - d2 * 0.125 + d3 * 0.0625;
                d2 = d2 * 0.25 - d3 * 0.125;
                d3 *= 0.125;

                stepsize /= 2;
                flatness = d2.magnitude_sq() + d3.magnitude_sq();
            }

            // Go to lower resolution if we're really flat
            // and we aren't going to overshoot the end.
            // XXX: tol/32 is just a guess for when we are too flat.
            while flatness > 0.0
                && flatness < tol / 32.0
                && stepsize < AFD_ONE
                && times + 2 * stepsize <= AFD_ONE
            {
                // Apply L^(-1) to the curve. Decrease curve resolution.
                d1 = d1 * 2.0 + d2;
                d2 = d2 * 4.0 + d3 * 4.0;
                d3 *= 8.0;

                stepsize *= 2;
                flatness = d2.magnitude_sq() + d3.magnitude_sq();
            }

            // Forward differencing.
            d0 += d1;
            d1 += d2;
            d2 += d3;

            // Output a point.
            self.add_point(
                d0,
                dist_tol,
                if times > 0 {
                    PointFlags::CORNER
                } else {
                    PointFlags::empty()
                },
            );

            // Advance along the curve.
            times += stepsize;

            // Ensure we don't overshoot.
            debug_assert!(times <= AFD_ONE);
        }
    }
}

#[inline]
fn curve_divisions(r: f32, arc: f32, tol: f32) -> usize {
    let da = (r / (r + tol)).acos() * 2.0;
    ((arc / da).ceil() as usize).max(2)
}

#[inline]
fn choose_bevel(bevel: bool, p0: &Point, p1: &Point, w: f32) -> [Offset; 2] {
    if bevel {
        let a = Offset::new(p1.pos.x + p0.dir.y * w, p1.pos.y - p0.dir.x * w);
        let b = Offset::new(p1.pos.x + p1.dir.y * w, p1.pos.y - p1.dir.x * w);
        [a, b]
    } else {
        [p1.pos + p1.ext * w, p1.pos + p1.ext * w]
    }
}

fn round_join(batch: &mut Batch, p0: &Point, p1: &Point, [lw, rw, lu, ru]: [f32; 4], ncap: usize) {
    let dl0 = Offset::new(p0.dir.y, -p0.dir.x);
    let dl1 = Offset::new(p1.dir.y, -p1.dir.x);

    if p1.flags.contains(PointFlags::LEFT) {
        let [l0, l1] = choose_bevel(p1.flags.contains(PointFlags::INNERBEVEL), p0, p1, lw);
        let a0 = f32::atan2(-dl0.y, -dl0.x);
        let a1 = f32::atan2(-dl1.y, -dl1.x);
        let a1 = if a1 > a0 { a1 - TAU } else { a1 };

        batch.push(Vertex::new(l0, [lu, 1.0]));
        batch.push(Vertex::new(p1.pos - dl0 * rw, [ru, 1.0]));

        let n = (((a0 - a1) / PI * ncap as f32).ceil() as usize).clamp(2, ncap as usize);
        for i in 0..n {
            let u = i as f32 / (n - 1) as f32;
            let a = a0 + u * (a1 - a0);
            let (sn, cs) = a.sin_cos();
            batch.push(Vertex::new(p1.pos, [0.5, 1.0]));
            batch.push(Vertex::new(p1.pos + Offset::new(cs, sn) * rw, [ru, 1.0]));
        }

        batch.push(Vertex::new(l1, [lu, 1.0]));
        batch.push(Vertex::new(p1.pos - dl1 * rw, [ru, 1.0]));
    } else {
        let [r0, r1] = choose_bevel(p1.flags.contains(PointFlags::INNERBEVEL), p0, p1, -rw);
        let a0 = f32::atan2(dl0.y, dl0.x);
        let a1 = f32::atan2(dl1.y, dl1.x);
        let a1 = if a1 < a0 { a1 + PI * 2.0 } else { a1 };

        batch.push(Vertex::new(p1.pos + dl0 * rw, [lu, 1.0]));
        batch.push(Vertex::new(r0, [ru, 1.0]));

        let n = (((a1 - a0) / PI * ncap as f32).ceil() as usize).clamp(2, ncap as usize);
        for i in 0..n {
            let u = i as f32 / (n - 1) as f32;
            let a = a0 + u * (a1 - a0);
            let (sn, cs) = a.sin_cos();
            batch.push(Vertex::new(p1.pos + Offset::new(cs, sn) * lw, [lu, 1.0]));
            batch.push(Vertex::new(p1.pos, [0.5, 1.0]));
        }

        batch.push(Vertex::new(p1.pos + dl1 * rw, [lu, 1.0]));
        batch.push(Vertex::new(r1, [ru, 1.0]));
    }
}

fn bevel_join(batch: &mut Batch, p0: &Point, p1: &Point, [lw, rw, lu, ru]: [f32; 4]) {
    let dl0 = Offset::new(p0.dir.y, -p0.dir.x);
    let dl1 = Offset::new(p1.dir.y, -p1.dir.x);

    if p1.flags.contains(PointFlags::LEFT) {
        let [l0, l1] = choose_bevel(p1.flags.contains(PointFlags::INNERBEVEL), p0, p1, lw);

        batch.push(Vertex::new(l0, [lu, 1.0]));
        batch.push(Vertex::new(p1.pos - dl0 * rw, [ru, 1.0]));

        if p1.flags.contains(PointFlags::BEVEL) {
            batch.push(Vertex::new(l0, [lu, 1.0]));
            batch.push(Vertex::new(p1.pos - dl0 * rw, [ru, 1.0]));

            batch.push(Vertex::new(l1, [lu, 1.0]));
            batch.push(Vertex::new(p1.pos - dl1 * rw, [ru, 1.0]));
        } else {
            let r0 = p1.pos - p1.ext * rw;

            batch.push(Vertex::new(p1.pos, [0.5, 1.0]));
            batch.push(Vertex::new(p1.pos - dl0 * rw, [ru, 1.0]));

            batch.push(Vertex::new(r0, [ru, 1.0]));
            batch.push(Vertex::new(r0, [ru, 1.0]));

            batch.push(Vertex::new(p1.pos, [0.5, 1.0]));
            batch.push(Vertex::new(p1.pos - dl1 * rw, [ru, 1.0]));
        }

        batch.push(Vertex::new(l1, [lu, 1.0]));
        batch.push(Vertex::new(p1.pos - dl1 * rw, [ru, 1.0]));
    } else {
        let [r0, r1] = choose_bevel(p1.flags.contains(PointFlags::INNERBEVEL), p0, p1, -rw);

        batch.push(Vertex::new(p1.pos + dl0 * lw, [lu, 1.0]));
        batch.push(Vertex::new(r0, [ru, 1.0]));

        if p1.flags.contains(PointFlags::BEVEL) {
            batch.push(Vertex::new(p1.pos + dl0 * lw, [lu, 1.0]));
            batch.push(Vertex::new(r0, [ru, 1.0]));

            batch.push(Vertex::new(p1.pos + dl1 * lw, [lu, 1.0]));
            batch.push(Vertex::new(r1, [ru, 1.0]));
        } else {
            let l0 = p1.pos + p1.ext * lw;

            batch.push(Vertex::new(p1.pos + dl0 * lw, [lu, 1.0]));
            batch.push(Vertex::new(p1.pos, [0.5, 1.0]));

            batch.push(Vertex::new(l0, [lu, 1.0]));
            batch.push(Vertex::new(l0, [lu, 1.0]));

            batch.push(Vertex::new(p1.pos + dl1 * lw, [lu, 1.0]));
            batch.push(Vertex::new(p1.pos, [0.5, 1.0]));
        }

        batch.push(Vertex::new(p1.pos + dl1 * lw, [lu, 1.0]));
        batch.push(Vertex::new(r1, [ru, 1.0]));
    }
}

fn butt_cap_start(
    batch: &mut Batch,
    p0: &Point,
    p1: &Point,
    w: f32,
    d: f32,
    aa: f32,
    u: (f32, f32),
) {
    let p = p0.pos - p1.dir * d;
    let dl = Offset::new(p1.dir.y, -p1.dir.x);

    batch.push(Vertex::new(p + dl * w - p1.dir * aa, [u.0, 0.0]));
    batch.push(Vertex::new(p - dl * w - p1.dir * aa, [u.1, 0.0]));
    batch.push(Vertex::new(p + dl * w, [u.0, 1.0]));
    batch.push(Vertex::new(p - dl * w, [u.1, 1.0]));
}

fn butt_cap_end(batch: &mut Batch, p0: &Point, p1: &Point, w: f32, d: f32, aa: f32, u: (f32, f32)) {
    let p = p0.pos + p1.dir * d;
    let dl = Offset::new(p1.dir.y, -p1.dir.x);

    batch.push(Vertex::new(p + dl * w, [u.0, 1.0]));
    batch.push(Vertex::new(p - dl * w, [u.1, 1.0]));
    batch.push(Vertex::new(p + dl * w + p1.dir * aa, [u.0, 0.0]));
    batch.push(Vertex::new(p - dl * w + p1.dir * aa, [u.1, 0.0]));
}

fn round_cap_start(batch: &mut Batch, p0: &Point, p1: &Point, w: f32, ncap: usize, u: (f32, f32)) {
    let p = p0.pos;
    let dl = Offset::new(p1.dir.y, -p1.dir.x);

    for i in 0..ncap {
        let angle = i as f32 / (ncap as f32 - 1.0) * PI;
        let (sin, cos) = angle.sin_cos();

        batch.push(Vertex::new(p - dl * cos * w - p1.dir * sin * w, [u.0, 1.0]));
        batch.push(Vertex::new(p, [0.5, 1.0]));
    }

    batch.push(Vertex::new(p + dl * w, [u.0, 1.0]));
    batch.push(Vertex::new(p - dl * w, [u.1, 1.0]));
}

fn round_cap_end(batch: &mut Batch, p0: &Point, p1: &Point, w: f32, ncap: usize, u: (f32, f32)) {
    let p = p0.pos;
    let dl = Offset::new(p1.dir.y, -p1.dir.x);

    batch.push(Vertex::new(p + dl * w, [u.0, 1.0]));
    batch.push(Vertex::new(p - dl * w, [u.1, 1.0]));

    for i in 0..ncap {
        let angle = i as f32 / (ncap as f32 - 1.0) * PI;
        let (sin, cos) = angle.sin_cos();

        batch.push(Vertex::new(p, [0.5, 1.0]));
        batch.push(Vertex::new(p - dl * cos * w + p1.dir * sin * w, [u.0, 1.0]));
    }
}
