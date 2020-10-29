use crate::{
    math::{Offset, PartialClamp},
    paint::{LineCap, LineJoin},
    path::{PathCmd, Winding},
    picture::Vertex,
};
use std::{f32::consts::PI, slice::from_raw_parts_mut};

impl Offset {
    #[inline]
    fn approx_eq_eps(self, other: Self, epsilon: f32) -> bool {
        let x = (self.x - other.x).abs() < epsilon;
        let y = (self.y - other.y).abs() < epsilon;
        x && y
    }

    #[inline(always)]
    fn normalize_mut(&mut self) -> f32 {
        let xx = self.x * self.x;
        let yy = self.y * self.y;
        let d = (xx + yy).sqrt();
        if d > 1e-6 {
            let id = d.recip();
            self.x *= id;
            self.y *= id;
        }
        d
    }
}

#[inline]
fn poly_area(pts: &[Point]) -> f32 {
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
fn curve_divs(r: f32, arc: f32, tol: f32) -> usize {
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
    fn is_left(&self) -> bool {
        self.flags.contains(PointFlags::LEFT)
    }
    fn is_corner(&self) -> bool {
        self.flags.contains(PointFlags::CORNER)
    }
    fn is_bevel(&self) -> bool {
        self.flags.contains(PointFlags::BEVEL)
    }
    fn is_innerbevel(&self) -> bool {
        self.flags.contains(PointFlags::INNERBEVEL)
    }

    fn any_bevel(&self) -> bool {
        self.flags
            .intersects(PointFlags::BEVEL | PointFlags::INNERBEVEL)
    }
}

pub struct CPath {
    start: usize,
    end: usize,

    pub fill: Option<&'static mut [Vertex]>,
    pub stroke: Option<&'static mut [Vertex]>,

    nbevel: usize,
    winding: Winding,
    closed: bool,
    pub convex: bool,
}

impl Default for CPath {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            fill: None,
            stroke: None,

            closed: false,
            nbevel: 0,
            winding: Winding::CCW,
            convex: false,
        }
    }
}

impl CPath {
    #[inline]
    fn len(&self) -> usize {
        self.end - self.start
    }

    #[inline]
    fn range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }

    fn pts<'a>(&self, pts: &'a [Point], p0: usize, p1: usize) -> PairIter<'a> {
        PairIter::new(&pts[self.range()], p0, p1)
    }
    fn pts_fan<'a>(&self, pts: &'a [Point], p0: usize, p1: usize) -> PairIterFan<'a> {
        PairIterFan::new(&pts[self.range()], p0, p1)
    }
}

struct PairIter<'a> {
    pts: &'a [Point],
    idx: usize,
    p0: usize,
    p1: usize,
}

impl<'a> PairIter<'a> {
    fn new(pts: &'a [Point], p0: usize, p1: usize) -> Self {
        let idx = 0;
        Self { pts, p0, p1, idx }
    }
}

impl<'a> Iterator for PairIter<'a> {
    type Item = (&'a Point, &'a Point);
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.pts.len() {
            None
        } else {
            let p0 = self.p0;
            let p1 = self.p1;
            let item = (&self.pts[p0], &self.pts[p1]);
            self.idx += 1;
            self.p0 = self.p1;
            self.p1 += 1;
            Some(item)
        }
    }
}

struct PairIterFan<'a> {
    pts: &'a [Point],
    idx: usize,
    p0: usize,
    p1: usize,
}

impl<'a> PairIterFan<'a> {
    fn new(pts: &'a [Point], p0: usize, p1: usize) -> Self {
        let idx = 0;
        Self { pts, p0, p1, idx }
    }
}

impl<'a> Iterator for PairIterFan<'a> {
    type Item = (&'a Point, &'a Point);
    fn next(&mut self) -> Option<Self::Item> {
        #[inline(always)]
        fn fan2strip(i: usize, len: usize) -> usize {
            if i % 2 != 0 {
                i / 2
            } else {
                len - 1 - i / 2
            }
        }

        if self.idx >= self.pts.len() {
            None
        } else {
            let p0 = fan2strip(self.p0, self.pts.len());
            let p1 = fan2strip(self.p1, self.pts.len());
            let item = (&self.pts[p0], &self.pts[p1]);
            self.idx += 1;
            self.p0 = self.p1;
            self.p1 += 1;
            Some(item)
        }
    }
}

pub struct Tessellator {
    points: Vec<Point>,
    last_point: Offset,

    pub paths: Vec<CPath>,
    pub verts: Vec<Vertex>,
    pub bounds: [f32; 4],

    pub tess_tol: f32,
    pub dist_tol: f32,
    pub fringe_width: f32,
}

impl Default for Tessellator {
    fn default() -> Self {
        Self {
            points: Vec::new(),
            last_point: Offset::zero(),

            paths: Vec::new(),
            verts: Vec::new(),
            bounds: [0.0; 4],

            tess_tol: 0.25,
            dist_tol: 0.01,
            fringe_width: 1.0,
        }
    }
}

impl Tessellator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.tess_tol = 0.25 / scale;
        self.dist_tol = 0.01 / scale;
        self.fringe_width = 1.0 / scale;
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.paths.clear();
        self.verts.clear();
        self.last_point = Offset::zero();
    }

    fn add_point(&mut self, point: Offset, dist_tol: f32, flags: PointFlags) {
        let path = match self.paths.last_mut() {
            Some(p) => p,
            None => return,
        };

        if path.len() > 0 {
            if let Some(pt) = self.points.last_mut() {
                if pt.pos.approx_eq_eps(point, dist_tol) {
                    pt.flags |= flags;
                    return;
                }
            }
        }

        self.points.push(Point {
            pos: point,
            flags,
            ..Default::default()
        });
        path.end += 1;

        self.last_point = point;
    }

    fn calculate_joins(&mut self, w: f32, line_join: LineJoin, miter_limit: f32) {
        let iw = if w > 0.0 { 1.0 / w } else { 0.0 };

        let miter_limit2 = miter_limit * miter_limit;

        // Calculate which joins needs extra vertices to append, and gather vertex count.
        for path in &mut self.paths {
            let pts = &mut self.points[path.range()];

            path.nbevel = 0;

            let mut p0_idx = path.len() - 1;
            let mut nleft = 0;
            for i in 0..pts.len() {
                let p0 = pts[p0_idx].clone();
                let p1 = &mut pts[i];

                let dl0 = Offset::new(p0.dir.y, -p0.dir.x);
                let dl1 = Offset::new(p1.dir.y, -p1.dir.x);

                // Calculate extrusions
                p1.ext = (dl0 + dl1) * 0.5;
                let dmr2 = p1.ext.magnitude_sq();
                if dmr2 > 0.000_001 {
                    p1.ext *= f32::min(1.0 / dmr2, 600.0);
                }

                // Clear flags, but keep the corner.
                p1.flags = if p1.is_corner() {
                    PointFlags::CORNER
                } else {
                    PointFlags::empty()
                };

                // Keep track of left turns.
                if p1.dir.cross(p0.dir) > 0.0 {
                    nleft += 1;
                    p1.flags |= PointFlags::LEFT;
                }

                // Calculate if we should use bevel or miter for inner join.
                let limit = f32::max(1.01, f32::min(p0.len, p1.len) * iw);
                if dmr2 * limit * limit < 1.0 {
                    p1.flags |= PointFlags::INNERBEVEL;
                }

                // Check to see if the corner needs to be beveled.
                if p1.is_corner() && (dmr2 * miter_limit2 < 1.0 || line_join != LineJoin::Miter) {
                    p1.flags |= PointFlags::BEVEL;
                }

                if p1.any_bevel() {
                    path.nbevel += 1;
                }

                p0_idx = i;
            }
            path.convex = nleft == path.len();
        }
    }

    pub fn flatten_paths(&mut self, commands: impl IntoIterator<Item = PathCmd>) {
        if !self.paths.is_empty() {
            return;
        }

        // Flatten
        for cmd in commands {
            match cmd {
                PathCmd::MoveTo(p) => {
                    let start = self.points.len();
                    self.paths.push(CPath {
                        start,
                        end: start,
                        ..Default::default()
                    });
                    self.add_point(p, self.dist_tol, PointFlags::CORNER);
                }
                PathCmd::LineTo(p) => self.add_point(p, self.dist_tol, PointFlags::CORNER),
                PathCmd::QuadTo(control, p3) => {
                    const FIX: f32 = 2.0 / 3.0;
                    let p0 = self.last_point;
                    let p1 = p0 + (control - p0) * FIX;
                    let p2 = p3 + (control - p3) * FIX;
                    self.tesselate_bezier(p0, p1, p2, p3, PointFlags::CORNER)
                }
                PathCmd::CubicTo(p1, p2, p3) => {
                    let p0 = self.last_point;
                    self.tesselate_bezier(p0, p1, p2, p3, PointFlags::CORNER);
                }
                PathCmd::Winding(dir) => {
                    if let Some(path) = self.paths.last_mut() {
                        path.winding = dir;
                    }
                }
                PathCmd::Close => {
                    if let Some(path) = self.paths.last_mut() {
                        path.closed = true;
                    }
                }
            }
        }

        self.bounds = [1e6, 1e6, -1e6, -1e6];

        // Calculate the direction and length of line segments.
        for path in &mut self.paths {
            let pts = &mut self.points[path.range()];
            assert!(pts.len() >= 2);

            // If the first and last points are the same, remove the last, mark as closed path.

            let last = pts.len() - 1;
            if pts[last].pos.approx_eq_eps(pts[0].pos, self.dist_tol) {
                path.end -= 1;
                path.closed = true;
            }

            let pts = &mut self.points[path.range()];

            // Enforce winding.
            if path.len() > 2 {
                let area = poly_area(&pts[..path.len()]);
                if path.winding == Winding::CCW && area < 0.0 {
                    pts.reverse();
                }
                if path.winding == Winding::CW && area > 0.0 {
                    pts.reverse();
                }
            }

            let mut p0 = path.len() - 1;
            for p1 in 0..path.len() {
                // Calculate segment direction and length
                pts[p0].dir = pts[p1].pos - pts[p0].pos;
                pts[p0].len = pts[p0].dir.normalize_mut();
                let pos = pts[p0].pos;
                // Update bounds
                self.bounds = [
                    self.bounds[0].min(pos.x),
                    self.bounds[1].min(pos.y),
                    self.bounds[2].max(pos.x),
                    self.bounds[3].max(pos.y),
                ];
                // Advance
                p0 = p1;
            }
        }
    }

    // Adaptive forward differencing for bezier tesselation.
    // See Lien, Sheue-Ling, Michael Shantz, and Vaughan Pratt.
    // "Adaptive forward differencing for rendering curves and surfaces."
    // ACM SIGGRAPH Computer Graphics. Vol. 21. No. 4. ACM, 1987.
    fn tesselate_bezier(
        &mut self,
        p0: Offset,
        p1: Offset,
        p2: Offset,
        p3: Offset,
        kind: PointFlags,
    ) {
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
        let tol = self.tess_tol * 4.0;
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
            let flags = if times > 0 { kind } else { PointFlags::empty() };
            self.add_point(d0, self.dist_tol, flags);
            // Advance along the curve.
            times += stepsize;
            // Ensure we don't overshoot.
            debug_assert!(times <= AFD_ONE);
        }
    }

    pub fn expand_stroke(
        &mut self,
        w: f32,
        line_cap: LineCap,
        line_join: LineJoin,
        miter_limit: f32,
    ) {
        let aa = self.fringe_width;
        let ncap = curve_divs(w, PI, self.tess_tol); // Calculate divisions per half circle.

        let w = w + aa * 0.5;

        // Disable the gradient used for antialiasing when antialiasing is not used.
        let u = if aa == 0.0 { (0.5, 0.5) } else { (0.0, 1.0) };

        self.calculate_joins(w, line_join, miter_limit);

        // Calculate max vertex usage.
        let mut additional = 0;
        for path in &self.paths {
            let count = if line_join == LineJoin::Round {
                ncap + 2
            } else {
                5
            };
            additional += (path.len() + path.nbevel * count + 1) * 2; // plus one for loop
            if !path.closed {
                // space for caps
                if line_cap == LineCap::Round {
                    additional += (ncap * 2 + 2) * 2;
                } else {
                    additional += (3 + 3) * 2;
                }
            }
        }
        self.verts.reserve(additional);

        for path in &mut self.paths {
            // Calculate fringe or stroke
            let looped = path.closed;

            let start = self.verts.len();
            let dst = &mut self.verts;

            let pts = &self.points[path.range()];
            let last_idx = path.len() - 1;

            let (mut p0_idx, mut p1_idx, range);
            if looped {
                // Looping
                p0_idx = last_idx;
                p1_idx = 0;
                range = 0..path.len();
            } else {
                // Add cap
                p0_idx = 0;
                p1_idx = 1;
                range = 1..path.len() - 1;

                let (p0, p1) = (&pts[p0_idx], &pts[p1_idx]);
                let delta: Offset = (p1.pos - p0.pos).normalize();
                match line_cap {
                    LineCap::Butt => dst.butt_cap_start(p0, delta, w, -aa * 0.5, aa, u),
                    LineCap::Square => dst.butt_cap_start(p0, delta, w, w - aa, aa, u),
                    LineCap::Round => dst.round_cap_start(p0, delta, w, ncap, aa, u),
                };
            }

            for _ in range {
                let (p0, p1) = (&pts[p0_idx], &pts[p1_idx]);

                if p1.any_bevel() {
                    if line_join == LineJoin::Round {
                        dst.round_join(p0, p1, [w, w, u.0, u.1], ncap);
                    } else {
                        dst.bevel_join(p0, p1, [w, w, u.0, u.1]);
                    }
                } else {
                    dst.add(p1.pos + p1.ext * w, [u.0, 1.0]);
                    dst.add(p1.pos - p1.ext * w, [u.1, 1.0]);
                }

                p0_idx = p1_idx;
                p1_idx += 1;
            }

            if looped {
                // Loop it
                let (v0, v1) = (dst[start].pos, dst[start + 1].pos);
                dst.add(v0.into(), [u.0, 1.0]);
                dst.add(v1.into(), [u.1, 1.0]);
            } else {
                // Add cap
                let (p0, p1) = (&pts[p0_idx], &pts[p1_idx % pts.len()]); // XXX
                let delta: Offset = (p1.pos - p0.pos).normalize();
                match line_cap {
                    LineCap::Butt => dst.butt_cap_end(p1, delta, w, -aa * 0.5, aa, u),
                    LineCap::Square => dst.butt_cap_end(p1, delta, w, w - aa, aa, u),
                    LineCap::Round => dst.round_cap_end(p1, delta, w, ncap as i32, aa, u),
                }
            }

            path.fill = None;
            path.stroke = Some(unsafe {
                let slice = &mut self.verts[start..];
                from_raw_parts_mut(slice.as_mut_ptr(), slice.len())
            });
        }
    }

    pub fn expand_fill(&mut self, w: f32, line_join: LineJoin, miter_limit: f32) {
        let woff = 0.5 * self.fringe_width;
        let has_fringe = w > 0.0;

        self.calculate_joins(w, line_join, miter_limit);

        // Calculate max vertex usage.
        let mut additional = 0;
        for path in &self.paths {
            additional += path.len() + path.nbevel + 1;
            if has_fringe {
                additional += (path.len() + path.nbevel * 5 + 1) * 2; // plus one for loop
            }
        }
        self.verts.reserve(additional);

        let convex = self.paths.len() == 1 && self.paths[0].convex;

        for path in &mut self.paths {
            let pts = &self.points[path.range()];
            let last_idx = path.len() - 1;

            // Calculate shape vertices.

            let start = self.verts.len();
            let dst = &mut self.verts;

            if has_fringe {
                // Looping
                for (p0, p1) in path.pts_fan(&self.points, last_idx, 0) {
                    if p1.is_bevel() && !p1.is_left() {
                        let dl0 = p1.pos + Offset::new(p0.dir.y, -p0.dir.x) * woff;
                        let dl1 = p1.pos + Offset::new(p1.dir.y, -p1.dir.x) * woff;
                        dst.add(dl0, [0.5, 1.0]);
                        dst.add(dl1, [0.5, 1.0]);
                    } else {
                        dst.add(p1.pos + p1.ext * woff, [0.5, 1.0]);
                    }
                }

                path.fill = Some(unsafe {
                    let slice = &mut self.verts[start..];
                    from_raw_parts_mut(slice.as_mut_ptr(), slice.len())
                });

                // Calculate fringe

                // Create only half a fringe for convex shapes so that
                // the shape can be rendered without stenciling.

                let (rw, ru) = (w - woff, 1.0);
                let (lw, lu) = if convex {
                    // This should generate the same vertex as fill inset above.
                    // Set outline fade at middle.
                    (woff, 0.5)
                } else {
                    (w + woff, 0.0)
                };

                let start = self.verts.len();
                let dst = &mut self.verts;

                // Looping
                for (p0, p1) in path.pts(&self.points, last_idx, 0) {
                    if p1.any_bevel() {
                        dst.bevel_join(p0, p1, [lw, rw, lu, ru]);
                    } else {
                        dst.add(p1.pos + p1.ext * lw, [lu, 1.0]);
                        dst.add(p1.pos - p1.ext * rw, [ru, 1.0]);
                    }
                }

                // Loop it
                let (v0, v1) = (dst[start].pos, dst[start + 1].pos);
                dst.add(v0.into(), [lu, 1.0]);
                dst.add(v1.into(), [ru, 1.0]);

                path.stroke = Some(unsafe {
                    let slice = &mut self.verts[start..];
                    from_raw_parts_mut(slice.as_mut_ptr(), slice.len())
                });
            } else {
                let first = &pts[0];
                for p in &pts[1..] {
                    dst.add(first.pos, [0.5, 1.0]);
                    dst.add(p.pos, [0.5, 1.0]);
                }

                path.fill = Some(unsafe {
                    let slice = &mut self.verts[start..];
                    from_raw_parts_mut(slice.as_mut_ptr(), slice.len())
                });

                path.stroke = None;
            }
        }
    }
}

trait Tess {
    fn add(&mut self, pos: Offset, uv: [f32; 2]);

    fn round_join(&mut self, p0: &Point, p1: &Point, [lw, rw, lu, ru]: [f32; 4], ncap: usize) {
        let dl0 = Offset::new(p0.dir.y, -p0.dir.x);
        let dl1 = Offset::new(p1.dir.y, -p1.dir.x);

        if p1.is_left() {
            let [l0, l1] = choose_bevel(p1.is_innerbevel(), p0, p1, lw);
            let a0 = (-dl0.y).atan2(-dl0.x);
            let a1 = (-dl1.y).atan2(-dl1.x);
            let a1 = if a1 > a0 { a1 - PI * 2.0 } else { a1 };

            self.add(l0, [lu, 1.0]);
            self.add(p1.pos - dl0 * rw, [ru, 1.0]);

            let n = (a0 - a1) / PI;
            let n = ((n * ncap as f32).ceil() as i32).clamp(2, ncap as i32);
            for i in 0..n {
                let u = (i as f32) / (n - 1) as f32;
                let a = a0 + u * (a1 - a0);
                let (sn, cs) = a.sin_cos();
                self.add(p1.pos, [0.5, 1.0]);
                self.add(p1.pos + Offset::new(cs, sn) * rw, [ru, 1.0]);
            }

            self.add(l1, [lu, 1.0]);
            self.add(p1.pos - dl1 * rw, [ru, 1.0]);
        } else {
            let [r0, r1] = choose_bevel(p1.is_innerbevel(), p0, p1, -rw);
            let a0 = f32::atan2(dl0.y, dl0.x);
            let a1 = f32::atan2(dl1.y, dl1.x);
            let a1 = if a1 < a0 { a1 + PI * 2.0 } else { a1 };

            self.add(p1.pos + dl0 * rw, [lu, 1.0]);
            self.add(r0, [ru, 1.0]);

            let n = (a1 - a0) / PI;
            let n = ((n * ncap as f32).ceil() as i32).clamp(2, ncap as i32);
            for i in 0..n {
                let u = (i as f32) / (n - 1) as f32;
                let a = a0 + u * (a1 - a0);
                let (sn, cs) = a.sin_cos();
                self.add(p1.pos + Offset::new(cs, sn) * lw, [lu, 1.0]);
                self.add(p1.pos, [0.5, 1.0]);
            }

            self.add(p1.pos + dl1 * rw, [lu, 1.0]);
            self.add(r1, [ru, 1.0]);
        }
    }

    fn bevel_join(&mut self, p0: &Point, p1: &Point, [lw, rw, lu, ru]: [f32; 4]) {
        let dl0 = Offset::new(p0.dir.y, -p0.dir.x);
        let dl1 = Offset::new(p1.dir.y, -p1.dir.x);

        if p1.is_left() {
            let [l0, l1] = choose_bevel(p1.is_innerbevel(), p0, p1, lw);

            self.add(l0, [lu, 1.0]);
            self.add(p1.pos - dl0 * rw, [ru, 1.0]);

            if p1.is_bevel() {
                self.add(l0, [lu, 1.0]);
                self.add(p1.pos - dl0 * rw, [ru, 1.0]);

                self.add(l1, [lu, 1.0]);
                self.add(p1.pos - dl1 * rw, [ru, 1.0]);
            } else {
                let r0 = p1.pos - p1.ext * rw;

                self.add(p1.pos, [0.5, 1.0]);
                self.add(p1.pos - dl0 * rw, [ru, 1.0]);

                self.add(r0, [ru, 1.0]);
                self.add(r0, [ru, 1.0]);

                self.add(p1.pos, [0.5, 1.0]);
                self.add(p1.pos - dl1 * rw, [ru, 1.0]);
            }

            self.add(l1, [lu, 1.0]);
            self.add(p1.pos - dl1 * rw, [ru, 1.0]);
        } else {
            let [r0, r1] = choose_bevel(p1.is_innerbevel(), p0, p1, -rw);

            self.add(p1.pos + dl0 * lw, [lu, 1.0]);
            self.add(r0, [ru, 1.0]);

            if p1.is_bevel() {
                self.add(p1.pos + dl0 * lw, [lu, 1.0]);
                self.add(r0, [ru, 1.0]);

                self.add(p1.pos + dl1 * lw, [lu, 1.0]);
                self.add(r1, [ru, 1.0]);
            } else {
                let l0 = p1.pos + p1.ext * lw;

                self.add(p1.pos + dl0 * lw, [lu, 1.0]);
                self.add(p1.pos, [0.5, 1.0]);

                self.add(l0, [lu, 1.0]);
                self.add(l0, [lu, 1.0]);

                self.add(p1.pos + dl1 * lw, [lu, 1.0]);
                self.add(p1.pos, [0.5, 1.0]);
            }

            self.add(p1.pos + dl1 * lw, [lu, 1.0]);
            self.add(r1, [ru, 1.0]);
        }
    }

    fn butt_cap_start(&mut self, p: &Point, delta: Offset, w: f32, d: f32, aa: f32, u: (f32, f32)) {
        let p1 = p.pos - delta * d;
        let p0 = p1 - delta * aa;
        let dl = Offset::new(delta.y, -delta.x) * w;
        self.add(p0 + dl, [u.0, 0.0]);
        self.add(p0 - dl, [u.1, 0.0]);
        self.add(p1 + dl, [u.0, 1.0]);
        self.add(p1 - dl, [u.1, 1.0]);
    }

    fn butt_cap_end(&mut self, p: &Point, delta: Offset, w: f32, d: f32, aa: f32, u: (f32, f32)) {
        let p1 = p.pos + delta * d;
        let p0 = p1 + delta * aa;
        let dl = Offset::new(delta.y, -delta.x) * w;
        self.add(p1 + dl, [u.0, 1.0]);
        self.add(p1 - dl, [u.1, 1.0]);
        self.add(p0 + dl, [u.0, 0.0]);
        self.add(p0 - dl, [u.1, 0.0]);
    }

    fn round_cap_start(
        &mut self,
        p: &Point,
        delta: Offset,
        w: f32,
        ncap: usize,
        _aa: f32,
        u: (f32, f32),
    ) {
        let (delta, dl) = (delta * w, Offset::new(delta.y, -delta.x) * w);
        for i in 0..ncap {
            let angle = (i as f32) / ((ncap - 1) as f32) * PI;
            let (sin, cos) = angle.sin_cos();
            self.add(p.pos - dl * cos - delta * sin, [u.0, 1.0]);
            self.add(p.pos, [0.5, 1.0]);
        }

        self.add(p.pos + dl, [u.0, 1.0]);
        self.add(p.pos - dl, [u.1, 1.0]);
    }

    fn round_cap_end(
        &mut self,
        p: &Point,
        delta: Offset,
        w: f32,
        ncap: i32,
        _aa: f32,
        u: (f32, f32),
    ) {
        let (delta, dl) = (delta * w, Offset::new(delta.y, -delta.x) * w);

        self.add(p.pos + dl, [u.0, 1.0]);
        self.add(p.pos - dl, [u.1, 1.0]);

        for i in 0..ncap {
            let angle = (i as f32) / ((ncap - 1) as f32) * PI;
            let (sin, cos) = angle.sin_cos();
            self.add(p.pos, [0.5, 1.0]);
            self.add(p.pos - dl * cos + delta * sin, [u.0, 1.0]);
        }
    }
}

impl Tess for Vec<Vertex> {
    #[inline(always)]
    fn add(&mut self, pos: Offset, uv: [f32; 2]) {
        Vec::push(self, Vertex::new(pos.into(), uv))
    }
}
