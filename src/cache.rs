#![allow(clippy::too_many_arguments)]

use std::{
    f32::consts::PI,
    ptr::null_mut,
    slice::from_raw_parts,
};

use euclid::vec2;

use crate::Vector;

use crate::vg::utils::{
    clampi,
    max,
    minf, maxf,
    normalize,
    pt_eq,
    vec_mul,
};

const INIT_POINTS_SIZE: usize = 128;
const INIT_PATHS_SIZE: usize = 16;
const INIT_VERTS_SIZE: usize = 256;

const MOVETO: i32 = 0;
const LINETO: i32 = 1;
const BEZIERTO: i32 = 2;
const CLOSE: i32 = 3;
const WINDING: i32 = 4;

fn triarea2(ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32) -> f32 {
    let abx = bx - ax;
    let aby = by - ay;
    let acx = cx - ax;
    let acy = cy - ay;
    acx*aby - abx*acy
}

fn poly_area(pts: &[Point]) -> f32 {
    let mut area = 0.0;
    let a = &pts[0];
    for i in 2..pts.len() {
        let b = &pts[i-1];
        let c = &pts[i];
        area += triarea2(a.x,a.y, b.x,b.y, c.x,c.y);
    }
    area * 0.5
}

fn curve_divs(r: f32, arc: f32, tol: f32) -> usize {
    let da = (r / (r + tol)).acos() * 2.0;
    max(2, (arc / da).ceil() as usize)
}

fn choose_bevel(bevel: bool, p0: &Point, p1: &Point, w: f32) -> [f32; 4] {
    if bevel {[
        p1.x + p0.dy * w,
        p1.y - p0.dx * w,
        p1.x + p1.dy * w,
        p1.y - p1.dx * w,
    ]} else {[
        p1.x + p1.dmx * w,
        p1.y + p1.dmy * w,
        p1.x + p1.dmx * w,
        p1.y + p1.dmy * w,
    ]}
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LineCap {
    Butt = 0,
    Round = 1,
    Square = 2,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum LineJoin {
    Round = 1,
    Bevel = 3,
    Miter = 4,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Winding {
    CCW = 1, // Winding for solid shapes
    CW = 2,  // Winding for holes
}

#[derive(Clone, Copy, Default)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [u16; 2],
}

impl Vertex {
    #[inline(always)]
    pub fn new(pos: [f32; 2], uv: [f32; 2]) -> Self {
        let uv = crate::vg::utils::pack_uv(uv[0], uv[1]);
        Self { pos, uv }
    }

    #[inline(always)]
    pub fn set(&mut self, pos: [f32; 2], uv: [f32; 2]) {
        *self = Self::new(pos, uv);
    }
}

bitflags::bitflags!(
    #[derive(Default)]
    pub struct PointFlags: u8 {
        const CORNER = 0x01;
        const LEFT = 0x02;
        const BEVEL = 0x04;
        const INNERBEVEL = 0x08;
    }
);

#[derive(Clone, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,

    pub dx: f32,
    pub dy: f32,

    pub len: f32,

    pub dmx: f32,
    pub dmy: f32,

    pub flags: PointFlags,
}

impl Point {
    pub fn is_left(&self) -> bool { self.flags.contains(PointFlags::LEFT) }
    pub fn is_corner(&self) -> bool { self.flags.contains(PointFlags::CORNER) }
    pub fn is_bevel(&self) -> bool { self.flags.contains(PointFlags::BEVEL) }
    pub fn is_innerbevel(&self) -> bool { self.flags.contains(PointFlags::INNERBEVEL) }
}

pub struct Path {
    pub first: usize,
    pub count: usize,

    pub closed: bool,
    pub nbevel: usize,

    pub fill: *mut Vertex,
    pub nfill: usize,

    pub stroke: *mut Vertex,
    pub nstroke: usize,

    pub winding: Winding,
    pub convex: bool,
}

impl Default for Path {
    fn default() -> Self {
        Self {
            first: 0,
            count: 0,
            closed: false,
            nbevel: 0,
            fill: null_mut(),
            nfill: 0,
            stroke: null_mut(),
            nstroke: 0,
            winding: Winding::CCW,
            convex: false,
        }
    }
}

impl Path {
    pub fn points<'a>(&self, pts: &'a [Point]) -> &'a [Point] {
        let start = self.first as usize;
        let len = self.count as usize;
        &pts[start..start+len]
    }
    pub fn points_mut<'a>(&self, pts: &'a mut [Point]) -> &'a mut [Point] {
        let start = self.first as usize;
        let len = self.count as usize;
        &mut pts[start..start+len]
    }

    fn pts<'a>(&self, pts: &'a [Point], p0: usize, p1: usize) -> PairIter<'a> {
        PairIter::new(self.points(pts), p0, p1)
    }

    /*
    fn pts_mut<'a>(&self, pts: &'a mut [Point], p0: usize, p1: usize) -> PairIterMut<'a> {
        PairIterMut::new(self.points_mut(pts), p0, p1)
    }
    */

    pub fn fill(&self) -> &[Vertex] {
        unsafe { from_raw_parts(self.fill, self.nfill) }
    }
    pub fn stroke(&self) -> &[Vertex] {
        unsafe { from_raw_parts(self.stroke, self.nstroke) }
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
        Self { pts, p0, p1, idx: 0 }
    }
}

impl<'a> Iterator for PairIter<'a> {
    type Item = (&'a Point, &'a Point);
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.pts.len() {
            None
        } else {
            let item = (&self.pts[self.p0], &self.pts[self.p1]);
            self.idx += 1;
            self.p0 = self.p1;
            self.p1 += 1;
            Some(item)
        }
    }
}

/*
struct PairIterMut<'a> {
    pts: &'a mut [Point],
    idx: usize,
    p0: usize,
    p1: usize,
}

impl<'a> PairIterMut<'a> {
    fn new(pts: &'a mut [Point], p0: usize, p1: usize) -> Self {
        Self { pts, p0, p1, idx: 0, }
    }
}

impl<'a> Iterator for PairIterMut<'a> {
    type Item = (&'a mut Point, &'a mut Point);
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx == self.pts.len() {
            None
        } else {
            let pts = self.pts.as_mut_ptr();
            let item = unsafe {
                let a = pts.add(self.p0);
                let b = pts.add(self.p1);
                (&mut *a, &mut *b)
            };
            self.idx += 1;
            self.p0 = self.p1;
            self.p1 += 1;
            Some(item)
        }
    }
}
*/

pub struct PathCache {
    points: Vec<Point>,
    pub paths: Vec<Path>,
    verts: Vec<Vertex>,
    pub bounds: [f32; 4],

    pub tess_tol: f32,
    pub dist_tol: f32,
    pub fringe_width: f32,
}

impl Default for PathCache {
    fn default() -> Self {
        Self {
            points: Vec::with_capacity(INIT_POINTS_SIZE),
            paths: Vec::with_capacity(INIT_PATHS_SIZE),
            verts: Vec::with_capacity(INIT_VERTS_SIZE),
            bounds: [0.0; 4],

            tess_tol: 0.25,
            dist_tol: 0.01,
            fringe_width: 1.0,
        }
    }
}

impl PathCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_dpi(&mut self, ratio: f32) {
        self.tess_tol = 0.25 / ratio;
        self.dist_tol = 0.01 / ratio;
        self.fringe_width = 1.0 / ratio;
    }

    pub fn clear(&mut self) {
	self.points.clear();
	self.paths.clear();
    }

    pub(crate) fn add_path(&mut self) {
        self.paths.push(Path {
            winding: Winding::CCW,
            first: self.points.len(),
            .. Default::default()
        });
    }

    pub(crate) fn add_point(&mut self, x: f32, y: f32, dist_tol: f32, flags: PointFlags) {
        let path = match self.paths.last_mut() {
            Some(p) => p,
            None => return,
        };

        if path.count as usize > 0 && !self.points.is_empty() {
            let pt = self.points.last_mut().expect("last point");
            if pt_eq(pt.x,pt.y, x,y, dist_tol) {
                pt.flags |= flags;
                return;
            }
        }

        self.points.push(Point { x, y, flags, .. Default::default() });

        path.count += 1;
    }

    pub(crate) fn resize_verts(&mut self, count: usize) {
        self.verts.resize_with(count, Default::default);
    }

    pub(crate) fn temp_verts(&mut self, count: usize) -> &mut [Vertex] {
        self.resize_verts(count);
        &mut self.verts[..count]
    }

    pub(crate) fn close_path(&mut self) {
        if let Some(path) = self.paths.last_mut() {
            path.closed = true;
        }
    }

    pub(crate) fn path_winding(&mut self, winding: Winding) {
        if let Some(path) = self.paths.last_mut() {
            path.winding = winding;
        }
    }

    pub(crate) fn calculate_joins(&mut self, w: f32, line_join: LineJoin, miter_limit: f32) {
        let iw = if w > 0.0 { 1.0 / w } else { 0.0 };

        // Calculate which joins needs extra vertices to append, and gather vertex count.
        for path in &mut self.paths {
            let pts = path.points_mut(&mut self.points);

            let last_idx = (path.count-1) as usize;
            let (mut p0_idx, mut p1_idx) = (last_idx, 0);

            path.nbevel = 0;

            let mut nleft = 0;
            for _ in 0..pts.len() {
                let p0 = pts[p0_idx].clone();
                let p1 = &mut pts[p1_idx];

                let dlx0 =  p0.dy;
                let dly0 = -p0.dx;
                let dlx1 =  p1.dy;
                let dly1 = -p1.dx;

                // Calculate extrusions
                p1.dmx = (dlx0 + dlx1) * 0.5;
                p1.dmy = (dly0 + dly1) * 0.5;
                let dmr2 = p1.dmx * p1.dmx + p1.dmy * p1.dmy;
                if dmr2 > 0.000_001 {
                    let scale = minf(1.0 / dmr2, 600.0);
                    p1.dmx *= scale;
                    p1.dmy *= scale;
                }

                // Clear flags, but keep the corner.
                p1.flags = if p1.is_corner() { PointFlags::CORNER } else { PointFlags::empty() };

                // Keep track of left turns.
                let cross = p1.dx * p0.dy - p0.dx * p1.dy;
                if cross > 0.0 {
                    nleft += 1;
                    p1.flags |= PointFlags::LEFT;
                }

                // Calculate if we should use bevel or miter for inner join.
                let limit = maxf(1.01, minf(p0.len, p1.len) * iw);
                if dmr2 * limit * limit < 1.0 {
                    p1.flags |= PointFlags::INNERBEVEL;
                }

                // Check to see if the corner needs to be beveled.
                if p1.is_corner() &&
                    (dmr2 * miter_limit * miter_limit < 1.0 || line_join != LineJoin::Miter)
                {
                    p1.flags |= PointFlags::BEVEL;
                }

                if p1.flags.intersects(PointFlags::BEVEL | PointFlags::INNERBEVEL) {
                    path.nbevel += 1;
                }

                p0_idx = p1_idx;
                p1_idx += 1;
            }
            path.convex = nleft == path.count;
        }
    }

    pub(crate) fn flatten_paths(&mut self, commands: &[f32]) {
        if !self.paths.is_empty() { return; }

        // Flatten
        let mut i = 0;
        while i < commands.len() {
            let cmd = &commands[i..];
            let kind = cmd[0] as i32;
            match kind {
            MOVETO => {
                self.add_path();
                let [x, y] = [cmd[1], cmd[2]];
                self.add_point(x, y, self.dist_tol, PointFlags::CORNER);
                i += 3;
            }
            LINETO => {
                let [x, y] = [cmd[1], cmd[2]];
                self.add_point(x, y, self.dist_tol, PointFlags::CORNER);
                i += 3;
            }
            BEZIERTO => {
                if let Some(last) = self.points.last() {
                    let p1 = [last.x, last.y];
                    let p2 = [cmd[1], cmd[2]];
                    let p3 = [cmd[3], cmd[4]];
                    let p4 = [cmd[5], cmd[6]];

                    self.tesselate_bezier(
                        p1, p2, p3, p4,
                        0, PointFlags::CORNER,
                    );
                }
                i += 7;
            }
            CLOSE => {
                self.close_path();
                i += 1;
            }
            WINDING => {
                self.path_winding(match cmd[1] as i32 {
                    1 => Winding::CCW,
                    2 => Winding::CW,
                    _ => unreachable!(),
                });
                i += 2;
            }
            //_ => { i += 1 }
            _ => unreachable!(),
            }
        }

        self.bounds = [1e6, 1e6, -1e6, -1e6];

        // Calculate the direction and length of line segments.
        for path in &mut self.paths {
            let pts = &mut self.points[path.first as usize..];

            // If the first and last points are the same, remove the last, mark as closed path.
            let mut p0: *mut Point = &mut pts[(path.count-1) as usize];
            let mut p1: *mut Point = &mut pts[0];
            if unsafe { pt_eq((*p0).x,(*p0).y, (*p1).x,(*p1).y, self.dist_tol) } {
                path.count -= 1;
                p0 = &mut pts[(path.count-1) as usize];
                path.closed = true;
            }

            let pts = path.points_mut(&mut self.points);

            // Enforce winding.
            if path.count > 2 {
                let area = poly_area(&pts[..path.count as usize]);
                if path.winding == Winding::CCW && area < 0.0 || path.winding == Winding::CW && area > 0.0 {
                    pts.reverse();
                }
            }

            for _ in 0..path.count {
                unsafe {
                    // Calculate segment direction and length
                    (*p0).dx = (*p1).x - (*p0).x;
                    (*p0).dy = (*p1).y - (*p0).y;
                    (*p0).len = normalize(&mut (*p0).dx, &mut (*p0).dy);
                    // Update bounds
                    self.bounds = [
                        minf(self.bounds[0], (*p0).x),
                        minf(self.bounds[1], (*p0).y),
                        maxf(self.bounds[2], (*p0).x),
                        maxf(self.bounds[3], (*p0).y),
                    ];
                    // Advance
                    p0 = p1;
                    p1 = p1.add(1);
                }
            }
        }
    }

    fn tesselate_bezier<P: Into<Vector>>(
        &mut self, p1: P, p2: P, p3: P, p4: P,
        level: i32, flags: PointFlags,
    ) {
        self.tesselate_bezier_impl(p1.into(), p2.into(), p3.into(), p4.into(), level, flags)
    }

    fn tesselate_bezier_impl(
        &mut self, p1: Vector, p2: Vector, p3: Vector, p4: Vector,
        level: i32, flags: PointFlags,
    ) {
        if level > 10 {
            return;
        }

        let [dx, dy]: [f32; 2] = (p4 - p1).into();

        let dp = Vector::new(dy, dx);

        let d2 = vec_mul(p2 - p4, dp);
        let d2 = (d2.x - d2.y).abs();

        let d3 = vec_mul(p3 - p4, dp);
        let d3 = (d3.x - d3.y).abs();

        if (d2 + d3)*(d2 + d3) < self.tess_tol * (dx*dx + dy*dy) {
            self.add_point(p4.x, p4.y, self.dist_tol, flags);
            return;
        }

        /*
        if false &&
            (p1.x+p3.x-p2.x-p2.x).abs() +
            (p1.y+p3.y-p2.y-p2.y).abs() +
            (p2.x+p4.x-p3.x-p3.x).abs() +
            (p2.y+p4.y-p3.y-p3.y).abs() < self.tess_tol {
            self.add_point(p4.x, p4.y, self.dist_tol, flags);
            return;
        }
        */

        let p12 = (p1 + p2) * 0.5;
        let p23 = (p2 + p3) * 0.5;
        let p34 = (p3 + p4) * 0.5;

        let p123 = (p12 + p23) * 0.5;
        let p234 = (p23 + p34) * 0.5;
        let p1234 = (p123 + p234) * 0.5;

        self.tesselate_bezier_impl(p1, p12, p123, p1234, level+1, PointFlags::empty());
        self.tesselate_bezier_impl(p1234, p234, p34, p4, level+1, flags);
    }

    pub fn expand_stroke(
        &mut self, w: f32, fringe: f32, line_cap: LineCap, line_join: LineJoin, miter_limit: f32,
    ) {
        let aa = fringe; //self.fringeWidth;
        let mut u0 = 0.0;
        let mut u1 = 1.0;
        let ncap = curve_divs(w, PI, self.tess_tol);    // Calculate divisions per half circle.

        let w = w + aa * 0.5;

        // Disable the gradient used for antialiasing when antialiasing is not used.
        if aa == 0.0 {
            u0 = 0.5;
            u1 = 0.5;
        }

        self.calculate_joins(w, line_join, miter_limit);

        // Calculate max vertex usage.
        let mut cverts = 0;
        for path in &self.paths {
            let count = if line_join == LineJoin::Round { ncap+2 } else { 5 };
            cverts += (path.count + path.nbevel*count + 1) * 2; // plus one for loop
            if !path.closed {
                // space for caps
                if line_cap == LineCap::Round {
                    cverts += (ncap*2 + 2)*2;
                } else {
                    cverts += (3+3)*2;
                }
            }
        }

        let mut verts = self.temp_verts(cverts as usize).as_mut_ptr();

        for path in &mut self.paths {
            path.fill = null_mut();
            path.nfill = 0;

            // Calculate fringe or stroke
            let looped = path.closed;

            path.stroke = verts;
            let mut dst = Verts::new(verts);

            let pts = path.points(&self.points);
            let last_idx = (path.count-1) as usize;

            let (mut p0_idx, mut p1_idx, s, e);
            if looped {
                // Looping
                p0_idx = last_idx;
                p1_idx = 0;
                s = 0;
                e = path.count;
            } else {
                // Add cap
                p0_idx = 0;
                p1_idx = 1;
                s = 1;
                e = path.count-1;
            }

            if !looped {
                // Add cap
                let (p0, p1) = (&pts[p0_idx], &pts[p1_idx]);
                let d: Vector = vec2(p1.x - p0.x, p1.y - p0.y).normalize();
                match line_cap {
                    LineCap::Butt => dst.butt_cap_start(p0, d.x, d.y, w, -aa*0.5, aa, u0, u1),
                    LineCap::Square => dst.butt_cap_start(p0, d.x, d.y, w, w-aa, aa, u0, u1),
                    LineCap::Round => dst.round_cap_start(p0, d.x, d.y, w, ncap as i32, aa, u0, u1),
                };
            }

            for _ in s..e {
                let (p0, p1) = (&pts[p0_idx], &pts[p1_idx]);

                if p1.flags.intersects(PointFlags::BEVEL | PointFlags::INNERBEVEL) {
                    if line_join == LineJoin::Round {
                        dst.round_join(p0, p1, w, w, u0, u1, ncap as i32, aa);
                    } else {
                        dst.bevel_join(p0, p1, w, w, u0, u1, aa);
                    }
                } else {
                    dst.push([p1.x + (p1.dmx * w), p1.y + (p1.dmy * w)], u0,1.0);
                    dst.push([p1.x - (p1.dmx * w), p1.y - (p1.dmy * w)], u1,1.0);
                }

                p0_idx = p1_idx;
                p1_idx += 1;
            }

            if looped {
                // Loop it
                let (v0, v1) = unsafe { (&*(verts.add(0)), &*(verts.add(1))) };
                dst.push(v0.pos, u0,1.0);
                dst.push(v1.pos, u1,1.0);
            } else {
                // Add cap
                let (p0, p1) = (&pts[p0_idx], &pts[p1_idx % pts.len()]); // XXX
                let d: Vector = vec2(p1.x - p0.x, p1.y - p0.y).normalize();
                match line_cap {
                    LineCap::Butt => dst.butt_cap_end(p1, d.x, d.y, w, -aa*0.5, aa, u0, u1),
                    LineCap::Square => dst.butt_cap_end(p1, d.x, d.y, w, w-aa, aa, u0, u1),
                    LineCap::Round => dst.round_cap_end(p1, d.x, d.y, w, ncap as i32, aa, u0, u1),
                };
            }

            path.nstroke = dst.count;
            verts = dst.dst;
        }
    }

    pub fn expand_fill(
        &mut self, w: f32, line_join: LineJoin, miter_limit: f32,
    ) {
        let aa = self.fringe_width;
        let fringe = w > 0.0;

        self.calculate_joins(w, line_join, miter_limit);

        // Calculate max vertex usage.
        let mut cverts = 0;
        for path in &self.paths {
            cverts += path.count + path.nbevel + 1;
            if fringe {
                cverts += (path.count + path.nbevel*5 + 1) * 2; // plus one for loop
            }
        }

        let mut verts = self.temp_verts(cverts as usize).as_mut_ptr();

        let convex = self.paths.len() == 1 && self.paths[0].convex;

        for path in &mut self.paths {
            let pts = path.points(&self.points);
            let last_idx = (path.count - 1) as usize;

            // Calculate shape vertices.
            let woff = 0.5*aa;

            path.fill = verts;
            let mut dst = Verts::new(verts);

            if fringe {
                // Looping
                for (p0, p1) in path.pts(&self.points, last_idx, 0) {
                    if p1.is_bevel() {
                        let dlx0 =  p0.dy;
                        let dly0 = -p0.dx;
                        let dlx1 =  p1.dy;
                        let dly1 = -p1.dx;
                        if p1.is_left() {
                            let lx = p1.x + p1.dmx * woff;
                            let ly = p1.y + p1.dmy * woff;
                            dst.push([lx, ly], 0.5,1.0);
                        } else {
                            let lx0 = p1.x + dlx0 * woff;
                            let ly0 = p1.y + dly0 * woff;
                            let lx1 = p1.x + dlx1 * woff;
                            let ly1 = p1.y + dly1 * woff;
                            dst.push([lx0, ly0], 0.5,1.0);
                            dst.push([lx1, ly1], 0.5,1.0);
                        }
                    } else {
                        dst.push([p1.x + (p1.dmx * woff), p1.y + (p1.dmy * woff)], 0.5,1.0);
                    }
                }
            } else {
                for p in pts {
                    dst.push([p.x, p.y], 0.5,1.0);
                }
            }

            path.nfill = dst.count;
            verts = dst.dst;

            // Calculate fringe
            if fringe {
                let mut lw = w + woff;
                let     rw = w - woff;
                let mut lu = 0.0;
                let     ru = 1.0;

                path.stroke = verts;
                let mut dst = Verts::new(verts);

                // Create only half a fringe for convex shapes so that
                // the shape can be rendered without stenciling.
                if convex {
                    lw = woff;  // This should generate the same vertex as fill inset above.
                    lu = 0.5;   // Set outline fade at middle.
                }

                // Looping
                for (p0, p1) in path.pts(&self.points, last_idx, 0) {
                    if p1.flags.intersects(PointFlags::BEVEL | PointFlags::INNERBEVEL) {
                        dst.bevel_join(p0, p1, lw, rw, lu, ru, self.fringe_width);
                    } else {
                        dst.push([p1.x + (p1.dmx * lw), p1.y + (p1.dmy * lw)], lu,1.0);
                        dst.push([p1.x - (p1.dmx * rw), p1.y - (p1.dmy * rw)], ru,1.0);
                    }
                }

                // Loop it
                let (v0, v1) = unsafe { (&*(verts.add(0)), &*(verts.add(1))) };
                dst.push(v0.pos, lu,1.0);
                dst.push(v1.pos, ru,1.0);

                path.nstroke = dst.count;
                verts = dst.dst;
            } else {
                path.stroke = null_mut();
                path.nstroke = 0;
            }
        }
    }
}

struct Verts {
    dst: *mut Vertex,
    count: usize,
}

impl Verts {
    fn new(dst: *mut Vertex) -> Self {
        Self { dst, count: 0 }
    }

    fn push(&mut self, pos: [f32; 2], u: f32, v: f32) {
        unsafe {
            *self.dst = Vertex::new(pos, [u, v]);
            self.dst = self.dst.add(1);
            self.count += 1;
        }
    }

    fn round_join(
        &mut self, p0: &Point, p1: &Point,
        lw: f32, rw: f32, lu: f32, ru: f32, ncap: i32, _fringe: f32,
    ) {
        let dlx0 =  p0.dy;
        let dly0 = -p0.dx;
        let dlx1 =  p1.dy;
        let dly1 = -p1.dx;

        if p1.is_left() {
            let [lx0,ly0,lx1,ly1] = choose_bevel(p1.is_innerbevel(), p0, p1, lw);
            let     a0 = (-dly0).atan2(-dlx0);
            let mut a1 = (-dly1).atan2(-dlx1);
            if a1 > a0 { a1 -= PI*2.0; }

            self.push([lx0, ly0], lu,1.0);
            self.push([p1.x - dlx0*rw, p1.y - dly0*rw], ru,1.0);

            let n = clampi((((a0 - a1) / PI) * ncap as f32).ceil() as i32, 2, ncap);
            for i in 0..n {
                let u = (i as f32) / (n-1) as f32;
                let a = a0 + u*(a1-a0);
                let rx = p1.x + a.cos() * rw;
                let ry = p1.y + a.sin() * rw;
                self.push([p1.x, p1.y], 0.5,1.0);
                self.push([rx, ry], ru,1.0);
            }

            self.push([lx1, ly1], lu,1.0);
            self.push([p1.x - dlx1*rw, p1.y - dly1*rw], ru,1.0);
        } else {
            let [rx0,ry0,rx1,ry1] = choose_bevel(p1.is_innerbevel(), p0, p1, -rw);
            let     a0 = (dly0).atan2(dlx0);
            let mut a1 = (dly1).atan2(dlx1);
            if a1 < a0 { a1 += PI*2.0; }

            self.push([p1.x + dlx0*rw, p1.y + dly0*rw], lu,1.0);
            self.push([rx0, ry0], ru,1.0);

            let n = clampi((((a1 - a0) / PI) * ncap as f32).ceil() as i32, 2, ncap);
            for i in 0..n {
                let u = (i as f32) / (n-1) as f32;
                let a = a0 + u*(a1-a0);
                let lx = p1.x + a.cos() * lw;
                let ly = p1.y + a.sin() * lw;
                self.push([lx, ly], lu,1.0);
                self.push([p1.x, p1.y], 0.5,1.0);
            }

            self.push([p1.x + dlx1*rw, p1.y + dly1*rw], lu,1.0);
            self.push([rx1, ry1], ru,1.0);
        }
    }

    fn bevel_join(
        &mut self, p0: &Point, p1: &Point,
        lw: f32, rw: f32, lu: f32, ru: f32, _fringe: f32,
    ) {
        let dlx0 =  p0.dy;
        let dly0 = -p0.dx;
        let dlx1 =  p1.dy;
        let dly1 = -p1.dx;

        if p1.is_left() {
            let [lx0, ly0,  lx1, ly1] = choose_bevel(p1.is_innerbevel(), p0, p1, lw);

            self.push([lx0, ly0], lu,1.0);
            self.push([p1.x - dlx0*rw, p1.y - dly0*rw], ru,1.0);

            if p1.is_bevel() {
                self.push([lx0, ly0], lu,1.0);
                self.push([p1.x - dlx0*rw, p1.y - dly0*rw], ru,1.0);

                self.push([lx1, ly1], lu,1.0);
                self.push([p1.x - dlx1*rw, p1.y - dly1*rw], ru,1.0);
            } else {
                let rx0 = p1.x - p1.dmx * rw;
                let ry0 = p1.y - p1.dmy * rw;

                self.push([p1.x, p1.y], 0.5,1.0);
                self.push([p1.x - dlx0*rw, p1.y - dly0*rw], ru,1.0);

                self.push([rx0, ry0], ru,1.0);
                self.push([rx0, ry0], ru,1.0);

                self.push([p1.x, p1.y], 0.5,1.0);
                self.push([p1.x - dlx1*rw, p1.y - dly1*rw], ru,1.0);
            }

            self.push([lx1, ly1], lu,1.0);
            self.push([p1.x - dlx1*rw, p1.y - dly1*rw], ru,1.0);
        } else {
            let [rx0, ry0, rx1, ry1] = choose_bevel(p1.is_innerbevel(), p0, p1, -rw);

            self.push([p1.x + dlx0*lw, p1.y + dly0*lw], lu,1.0);
            self.push([rx0, ry0], ru,1.0);

            if p1.is_bevel() {
                self.push([p1.x + dlx0*lw, p1.y + dly0*lw], lu,1.0);
                self.push([rx0, ry0], ru,1.0);

                self.push([p1.x + dlx1*lw, p1.y + dly1*lw], lu,1.0);
                self.push([rx1, ry1], ru,1.0);
            } else {
                let lx0 = p1.x + p1.dmx * lw;
                let ly0 = p1.y + p1.dmy * lw;

                self.push([p1.x + dlx0*lw, p1.y + dly0*lw], lu,1.0);
                self.push([p1.x, p1.y], 0.5,1.0);

                self.push([lx0, ly0], lu,1.0);
                self.push([lx0, ly0], lu,1.0);

                self.push([p1.x + dlx1*lw, p1.y + dly1*lw], lu,1.0);
                self.push([p1.x, p1.y], 0.5,1.0);
            }

            self.push([p1.x + dlx1*lw, p1.y + dly1*lw], lu,1.0);
            self.push([rx1, ry1], ru,1.0);
        }
    }

    fn butt_cap_start(
        &mut self, p: &Point,
        dx: f32, dy: f32, w: f32, d: f32,
        aa: f32, u0: f32, u1: f32,
    ) {
        let px = p.x - dx*d;
        let py = p.y - dy*d;
        let dlx = dy;
        let dly = -dx;
        self.push([px + dlx*w - dx*aa, py + dly*w - dy*aa], u0,0.0);
        self.push([px - dlx*w - dx*aa, py - dly*w - dy*aa], u1,0.0);
        self.push([px + dlx*w, py + dly*w], u0,1.0);
        self.push([px - dlx*w, py - dly*w], u1,1.0);
    }

    fn butt_cap_end(
        &mut self, p: &Point,
        dx: f32, dy: f32, w: f32, d: f32,
        aa: f32, u0: f32, u1: f32,
    ) {
        let px = p.x + dx*d;
        let py = p.y + dy*d;
        let dlx = dy;
        let dly = -dx;
        self.push([px + dlx*w, py + dly*w], u0,1.0);
        self.push([px - dlx*w, py - dly*w], u1,1.0);
        self.push([px + dlx*w + dx*aa, py + dly*w + dy*aa], u0,0.0);
        self.push([px - dlx*w + dx*aa, py - dly*w + dy*aa], u1,0.0);
    }

    fn round_cap_start(
        &mut self, p: &Point,
        dx: f32, dy: f32, w: f32, ncap: i32,
        _aa: f32, u0: f32, u1: f32,
    ) {
        let px = p.x;
        let py = p.y;
        let dlx = dy;
        let dly = -dx;
        for i in 0..ncap {
            let a = (i as f32) / ((ncap-1) as f32) * PI;
            let ax = a.cos() * w;
            let ay = a.sin() * w;
            self.push([px - dlx*ax - dx*ay, py - dly*ax - dy*ay], u0,1.0);
            self.push([px, py], 0.5,1.0);
        }
        self.push([px + dlx*w, py + dly*w], u0,1.0);
        self.push([px - dlx*w, py - dly*w], u1,1.0);
    }

    fn round_cap_end(
        &mut self, p: &Point,
        dx: f32, dy: f32, w: f32, ncap: i32,
        _aa: f32, u0: f32, u1: f32,
    ) {
        let px = p.x;
        let py = p.y;
        let dlx = dy;
        let dly = -dx;
        self.push([px + dlx*w, py + dly*w], u0,1.0);
        self.push([px - dlx*w, py - dly*w], u1,1.0);
        for i in 0..ncap {
            let a = (i as f32) / ((ncap-1) as f32) * PI;
            let ax = a.cos() * w;
            let ay = a.sin() * w;
            self.push([px, py], 0.5,1.0);
            self.push([px - dlx*ax + dx*ay, py - dly*ax + dy*ay], u0,1.0);
        }
    }
}
