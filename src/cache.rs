#![allow(clippy::too_many_arguments)]

use std::{
    f32::consts::PI,
    slice::from_raw_parts_mut,
};
use crate::{
    math::{Vector, vec2},
    vg::utils::{
        normalize,
        pt_eq,
        vec_mul,
    },
};

const INIT_POINTS_SIZE: usize = 128;
const INIT_PATHS_SIZE: usize = 16;
const INIT_VERTS_SIZE: usize = 256;

const MOVETO: i32 = 0;
const LINETO: i32 = 1;
const BEZIERTO: i32 = 2;
const CLOSE: i32 = 3;
const WINDING: i32 = 4;

#[inline(always)]
fn triarea2(a: Vector, b: Vector, c: Vector) -> f32 {
    (c - a).cross(b - a)
}

#[inline]
fn poly_area(pts: &[PathPoint]) -> f32 {
    let mut area = 0.0;
    let a = &pts[0];
    for i in 2..pts.len() {
        let b = &pts[i-1];
        let c = &pts[i];
        area += triarea2(a.pos, b.pos, c.pos);
    }
    area * 0.5
}

#[inline]
fn curve_divs(r: f32, arc: f32, tol: f32) -> usize {
    let da = (r / (r + tol)).acos() * 2.0;
    ((arc / da).ceil() as usize).max(2)
}

#[inline]
fn choose_bevel(bevel: bool, p0: &PathPoint, p1: &PathPoint, w: f32) -> [Vector; 2] {
    if bevel {[
        vec2(p1.pos.x + p0.dir.y * w, p1.pos.y - p0.dir.x * w),
        vec2(p1.pos.x + p1.dir.y * w, p1.pos.y - p1.dir.x * w),
    ]} else {[
        p1.pos + p1.ext * w,
        p1.pos + p1.ext * w,
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
struct PathPoint {
    // position
    pos: Vector,
    // direction
    dir: Vector,
    // extrusions
    ext: Vector,
    len: f32,
    flags: PointFlags,
}

impl PathPoint {
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

    pub fill: Option<&'static mut [Vertex]>,
    pub stroke: Option<&'static mut [Vertex]>,

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
            fill: None,
            stroke: None,
            winding: Winding::CCW,
            convex: false,
        }
    }
}

impl Path {
    fn points<'a>(&self, pts: &'a [PathPoint]) -> &'a [PathPoint] {
        let start = self.first as usize;
        let len = self.count as usize;
        &pts[start..start+len]
    }
    fn points_mut<'a>(&self, pts: &'a mut [PathPoint]) -> &'a mut [PathPoint] {
        let start = self.first as usize;
        let len = self.count as usize;
        &mut pts[start..start+len]
    }

    fn pts<'a>(&self, pts: &'a [PathPoint], p0: usize, p1: usize) -> PairIter<'a> {
        PairIter::new(self.points(pts), p0, p1)
    }
    fn pts_fan<'a>(&self, pts: &'a [PathPoint], p0: usize, p1: usize) -> PairIterFan<'a> {
        PairIterFan::new(self.points(pts), p0, p1)
    }

    /*
    fn pts_mut<'a>(&self, pts: &'a mut [Point], p0: usize, p1: usize) -> PairIterMut<'a> {
        PairIterMut::new(self.points_mut(pts), p0, p1)
    }
    */
}

struct PairIter<'a> {
    pts: &'a [PathPoint],
    idx: usize,
    p0: usize,
    p1: usize,
}

impl<'a> PairIter<'a> {
    fn new(pts: &'a [PathPoint], p0: usize, p1: usize) -> Self {
        Self { pts, p0, p1, idx: 0 }
    }
}

impl<'a> Iterator for PairIter<'a> {
    type Item = (&'a PathPoint, &'a PathPoint);
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
    pts: &'a [PathPoint],
    idx: usize,
    p0: usize,
    p1: usize,
}

impl<'a> PairIterFan<'a> {
    fn new(pts: &'a [PathPoint], p0: usize, p1: usize) -> Self {
        Self { pts, p0, p1, idx: 0 }
    }
}

impl<'a> Iterator for PairIterFan<'a> {
    type Item = (&'a PathPoint, &'a PathPoint);
    fn next(&mut self) -> Option<Self::Item> {
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

#[inline(always)]
fn fan2strip(i: usize, len: usize) -> usize {
    if i % 2 != 0 {
        i / 2
    } else {
        len - 1 - i / 2
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
    points: Vec<PathPoint>,
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
            if pt_eq(pt.pos.x,pt.pos.y, x,y, dist_tol) {
                pt.flags |= flags;
                return;
            }
        }

        self.points.push(PathPoint { pos: vec2(x, y), flags, .. Default::default() });

        path.count += 1;
    }

    pub(crate) fn temp_verts(&mut self, count: usize) -> &mut [Vertex] {
        self.verts.resize_with(count, Default::default);
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

    fn calculate_joins(&mut self, w: f32, line_join: LineJoin, miter_limit: f32) {
        let iw = if w > 0.0 { 1.0 / w } else { 0.0 };

        let miter_limit2 = miter_limit * miter_limit;

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

                let dl0 = vec2(p0.dir.y, -p0.dir.x);
                let dl1 = vec2(p1.dir.y, -p1.dir.x);

                // Calculate extrusions
                p1.ext = (dl0 + dl1) * 0.5;
                let dmr2 = p1.ext.square_length();
                if dmr2 > 0.000_001 {
                    let v = f32::min(1.0 / dmr2, 600.0);
                    p1.ext.x *= v;
                    p1.ext.y *= v;
                }

                // Clear flags, but keep the corner.
                p1.flags = if p1.is_corner() { PointFlags::CORNER } else { PointFlags::empty() };

                // Keep track of left turns.
                let cross = p1.dir.x * p0.dir.y - p0.dir.x * p1.dir.y;
                if cross > 0.0 {
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
                    let p1 = [last.pos.x, last.pos.y];
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
            let pts = &mut self.points[path.first..path.first + path.count];
            assert!(pts.len() >= 2);

            // If the first and last points are the same, remove the last, mark as closed path.
            let mut p0: *mut PathPoint = &mut pts[(path.count-1) as usize];
            let mut p1: *mut PathPoint = &mut pts[0];

            unsafe {
                if pt_eq((*p0).pos.x,(*p0).pos.y, (*p1).pos.x,(*p1).pos.y, self.dist_tol) {
                    path.count -= 1;
                    p0 = pts.get_unchecked_mut((path.count-1) as usize);
                    path.closed = true;
                }
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
                    (*p0).dir = (*p1).pos - (*p0).pos;
                    (*p0).len = normalize(&mut (*p0).dir.x, &mut (*p0).dir.y);
                    let pos = (*p0).pos;
                    // Update bounds
                    self.bounds = [
                        self.bounds[0].min(pos.x),
                        self.bounds[1].min(pos.y),
                        self.bounds[2].max(pos.x),
                        self.bounds[3].max(pos.y),
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

        let dp = (p4 - p1).yx();

        let d2 = vec_mul(p2 - p4, dp);
        let d2 = (d2.x - d2.y).abs();

        let d3 = vec_mul(p3 - p4, dp);
        let d3 = (d3.x - d3.y).abs();

        if (d2 + d3)*(d2 + d3) < self.tess_tol * dp.square_length() {
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
                    cverts += (ncap*2 + 2) * 2;
                } else {
                    cverts += (3+3) * 2;
                }
            }
        }

        let mut verts = self.temp_verts(cverts as usize).as_mut_ptr();

        for path in &mut self.paths {
            path.fill = None;

            // Calculate fringe or stroke
            let looped = path.closed;

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
                let d: Vector = (p1.pos - p0.pos).normalize();
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
                    dst.push(p1.pos + p1.ext * w, [u0,1.0]);
                    dst.push(p1.pos - p1.ext * w, [u1,1.0]);
                }

                p0_idx = p1_idx;
                p1_idx += 1;
            }

            if looped {
                // Loop it
                let (v0, v1) = (dst[0].pos, dst[1].pos);
                dst.push(v0.into(), [u0,1.0]);
                dst.push(v1.into(), [u1,1.0]);
            } else {
                // Add cap
                let (p0, p1) = (&pts[p0_idx], &pts[p1_idx % pts.len()]); // XXX
                let d: Vector = (p1.pos - p0.pos).normalize();
                match line_cap {
                    LineCap::Butt => dst.butt_cap_end(p1, d.x, d.y, w, -aa*0.5, aa, u0, u1),
                    LineCap::Square => dst.butt_cap_end(p1, d.x, d.y, w, w-aa, aa, u0, u1),
                    LineCap::Round => dst.round_cap_end(p1, d.x, d.y, w, ncap as i32, aa, u0, u1),
                };
            }

            path.stroke = Some(dst.raw_parts_mut());
            verts = dst.end_ptr();
        }
    }

    pub fn expand_fill(&mut self, w: f32, line_join: LineJoin, miter_limit: f32) {
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

            let mut dst = Verts::new(verts);

            if fringe {
                // Looping
                for (p0, p1) in path.pts_fan(&self.points, last_idx, 0) {
                    if p1.is_bevel() {
                        let dl0 = vec2(p0.dir.y, -p0.dir.x);
                        let dl1 = vec2(p1.dir.y, -p1.dir.x);
                        if p1.is_left() {
                            dst.push(p1.pos + p1.ext * woff, [0.5,1.0]);
                        } else {
                            dst.push(p1.pos + dl0 * woff, [0.5,1.0]);
                            dst.push(p1.pos + dl1 * woff, [0.5,1.0]);
                        }
                    } else {
                        dst.push(p1.pos + p1.ext * woff, [0.5,1.0]);
                    }
                }
            } else {
                for p in pts {
                    dst.push(p.pos, [0.5,1.0]);
                }
            }

            path.fill = Some(dst.raw_parts_mut());
            verts = dst.end_ptr();

            // Calculate fringe
            if fringe {
                let mut lw = w + woff;
                let     rw = w - woff;
                let mut lu = 0.0;
                let     ru = 1.0;

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
                        dst.push(p1.pos + p1.ext * lw, [lu,1.0]);
                        dst.push(p1.pos - p1.ext * rw, [ru,1.0]);
                    }
                }

                // Loop it
                let (v0, v1) = (dst[0].pos, dst[1].pos);
                dst.push(v0.into(), [lu,1.0]);
                dst.push(v1.into(), [ru,1.0]);

                path.stroke = Some(dst.raw_parts_mut());
                verts = dst.end_ptr();
            } else {
                path.stroke = None;
            }
        }
    }
}

struct Verts {
    start: *mut Vertex,
    count: usize,
}

impl std::ops::Index<usize> for Verts {
    type Output = Vertex;
    fn index(&self, idx: usize) -> &Self::Output {
        assert!(idx < self.count, "verts index: {}, len: {}", idx, self.count);
        unsafe {
            &*self.start.add(idx)
        }
    }
}

impl Verts {
    fn new(start: *mut Vertex) -> Self {
        Self { start, count: 0 }
    }

    fn end_ptr(self) -> *mut Vertex {
        unsafe { self.start.add(self.count) }
    }

    fn raw_parts_mut<'a>(&mut self) -> &'a mut [Vertex] {
        unsafe { from_raw_parts_mut(self.start, self.count) }
    }

    #[inline(always)]
    fn push(&mut self, pos: Vector, uv: [f32; 2]) {
        unsafe {
            *self.start.add(self.count) = Vertex::new(pos.into(), uv);
            self.count += 1;
        }
    }

    fn round_join(
        &mut self, p0: &PathPoint, p1: &PathPoint,
        lw: f32, rw: f32,
        lu: f32, ru: f32,
        ncap: i32,
        _fringe: f32,
    ) {
        let dl0 = vec2(p0.dir.y, -p0.dir.x);
        let dl1 = vec2(p1.dir.y, -p1.dir.x);

        if p1.is_left() {
            let [l0, l1] = choose_bevel(p1.is_innerbevel(), p0, p1, lw);
            let a0 = (-dl0.y).atan2(-dl0.x);
            let a1 = (-dl1.y).atan2(-dl1.x);
            let a1 = if a1 > a0 { a1 - PI*2.0 } else { a1 };

            self.push(l0, [lu,1.0]);
            self.push(p1.pos - dl0*rw, [ru,1.0]);

            let n = (a0 - a1) / PI;
            let n = ((n * ncap as f32).ceil() as i32).clamp(2, ncap);
            for i in 0..n {
                let u = (i as f32) / (n-1) as f32;
                let a = a0 + u*(a1-a0);
                let (sn, cs) = a.sin_cos();
                self.push(p1.pos, [0.5,1.0]);
                self.push(p1.pos + vec2(cs, sn) * rw, [ru,1.0]);
            }

            self.push(l1, [lu,1.0]);
            self.push(p1.pos - dl1*rw, [ru,1.0]);
        } else {
            let [r0, r1] = choose_bevel(p1.is_innerbevel(), p0, p1, -rw);
            let     a0 = (dl0.y).atan2(dl0.x);
            let mut a1 = (dl1.y).atan2(dl1.x);
            if a1 < a0 { a1 += PI*2.0; }

            self.push(p1.pos + dl0*rw, [lu,1.0]);
            self.push(r0, [ru,1.0]);

            let n = (a1 - a0) / PI;
            let n = ((n * ncap as f32).ceil() as i32).clamp(2, ncap);
            for i in 0..n {
                let u = (i as f32) / (n-1) as f32;
                let a = a0 + u*(a1-a0);
                let (sn, cs) = a.sin_cos();
                self.push(p1.pos + vec2(cs, sn) * lw, [lu,1.0]);
                self.push(p1.pos, [0.5,1.0]);
            }

            self.push(p1.pos + dl1*rw, [lu,1.0]);
            self.push(r1, [ru,1.0]);
        }
    }

    fn bevel_join(
        &mut self, p0: &PathPoint, p1: &PathPoint,
        lw: f32, rw: f32, lu: f32, ru: f32, _fringe: f32,
    ) {
        let dl0 = vec2(p0.dir.y, -p0.dir.x);
        let dl1 = vec2(p1.dir.y, -p1.dir.x);

        if p1.is_left() {
            let [l0, l1] = choose_bevel(p1.is_innerbevel(), p0, p1, lw);

            self.push(l0, [lu,1.0]);
            self.push(p1.pos - dl0*rw, [ru,1.0]);

            if p1.is_bevel() {
                self.push(l0, [lu,1.0]);
                self.push(p1.pos - dl0*rw, [ru,1.0]);

                self.push(l1, [lu,1.0]);
                self.push(p1.pos - dl1*rw, [ru,1.0]);
            } else {
                let r0 = p1.pos - p1.ext * rw;

                self.push(p1.pos, [0.5,1.0]);
                self.push(p1.pos - dl0*rw, [ru,1.0]);

                self.push(r0, [ru,1.0]);
                self.push(r0, [ru,1.0]);

                self.push(p1.pos, [0.5,1.0]);
                self.push(p1.pos - dl1*rw, [ru,1.0]);
            }

            self.push(l1, [lu,1.0]);
            self.push(p1.pos - dl1*rw, [ru,1.0]);
        } else {
            let [r0, r1] = choose_bevel(p1.is_innerbevel(), p0, p1, -rw);

            self.push(p1.pos + dl0*lw, [lu,1.0]);
            self.push(r0, [ru,1.0]);

            if p1.is_bevel() {
                self.push(p1.pos + dl0*lw, [lu,1.0]);
                self.push(r0, [ru,1.0]);

                self.push(p1.pos + dl1*lw, [lu,1.0]);
                self.push(r1, [ru,1.0]);
            } else {
                let l0 = p1.pos + p1.ext * lw;

                self.push(p1.pos + dl0*lw, [lu,1.0]);
                self.push(p1.pos, [0.5,1.0]);

                self.push(l0, [lu,1.0]);
                self.push(l0, [lu,1.0]);

                self.push(p1.pos + dl1*lw, [lu,1.0]);
                self.push(p1.pos, [0.5,1.0]);
            }

            self.push(p1.pos + dl1*lw, [lu,1.0]);
            self.push(r1, [ru,1.0]);
        }
    }

    fn butt_cap_start(
        &mut self, p: &PathPoint,
        dx: f32, dy: f32, w: f32, d: f32,
        aa: f32, u0: f32, u1: f32,
    ) {
        let dd = vec2(dx, dy);
        let p = p.pos - dd * d;
        let dl = vec2(dy, -dx) * w;
        self.push(p + dl - dd*aa, [u0,0.0]);
        self.push(p - dl - dd*aa, [u1,0.0]);
        self.push(p + dl, [u0,1.0]);
        self.push(p - dl, [u1,1.0]);
    }

    fn butt_cap_end(
        &mut self, p: &PathPoint,
        dx: f32, dy: f32, w: f32, d: f32,
        aa: f32, u0: f32, u1: f32,
    ) {
        let dd = vec2(dx, dy);
        let p = p.pos + dd * d;
        let dl = vec2(dy, -dx) * w;
        self.push(p + dl, [u0,1.0]);
        self.push(p - dl, [u1,1.0]);
        self.push(p + dl + dd * aa, [u0,0.0]);
        self.push(p - dl + dd * aa, [u1,0.0]);
    }

    fn round_cap_start(
        &mut self, p: &PathPoint,
        dx: f32, dy: f32, w: f32, ncap: i32,
        _aa: f32, u0: f32, u1: f32,
    ) {
        let dl = vec2(dy, -dx);
        for i in 0..ncap {
            let a = (i as f32) / ((ncap-1) as f32) * PI;
            let ax = a.cos() * w;
            let ay = a.sin() * w;
            self.push(p.pos - dl*ax - vec2(dx*ay, dy*ay), [u0,1.0]);
            self.push(p.pos, [0.5,1.0]);
        }
        self.push(p.pos + dl*w, [u0,1.0]);
        self.push(p.pos - dl*w, [u1,1.0]);
    }

    fn round_cap_end(
        &mut self, p: &PathPoint,
        dx: f32, dy: f32, w: f32, ncap: i32,
        _aa: f32, u0: f32, u1: f32,
    ) {
        let dl = vec2(dy, -dx);
        self.push(p.pos + dl*w, [u0,1.0]);
        self.push(p.pos - dl*w, [u1,1.0]);
        for i in 0..ncap {
            let a = (i as f32) / ((ncap-1) as f32) * PI;
            let (sn, cs) = a.sin_cos();
            let ax = cs * w;
            let ay = sn * w;
            self.push(p.pos, [0.5,1.0]);
            self.push(p.pos - dl*ax + vec2(dx*ay, dy*ay), [u0,1.0]);
        }
    }
}
