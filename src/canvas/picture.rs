use crate::{
    math::{
        Transform,
        Point, Vector,
        point2, vec2,
        Rect,
        ApproxEq,
    },
    cache::Winding,
};

#[inline(always)]
fn transform_pt(pt: &mut [f32], t: &Transform) {
    let p = t.transform_point(point2(pt[0], pt[1]));
    pt[0] = p.x;
    pt[1] = p.y;
}

fn dist_pt_seg(point: Vector, p: Vector, q: Vector) -> f32 {
    let pq = q - p;

    let len = pq.square_length();
    let mut t = pq.dot(point - p);
    if len > 0.0 { t /= len }

    (p + pq * t.clamp(0.0, 1.0) + point).square_length()
}

pub const MOVETO: u32 = 0;
pub const LINETO: u32 = 1;
pub const BEZIERTO: u32 = 2;
pub const CLOSE: u32 = 3;
pub const WINDING: u32 = 4;

fn transform_commands(commands: &mut [f32], xform: &Transform) {
    let mut i = 0;
    while i < commands.len() {
        let cmd = commands[i] as u32;
        match cmd {
        MOVETO => {
            transform_pt(&mut commands[i+1..], xform);
            i += 3;
        }
        LINETO => {
            transform_pt(&mut commands[i+1..], xform);
            i += 3;
        }
        BEZIERTO => {
            transform_pt(&mut commands[i+1..], xform);
            transform_pt(&mut commands[i+3..], xform);
            transform_pt(&mut commands[i+5..], xform);
            i += 7;
        }
        CLOSE => i += 1,
        WINDING => i += 2,
        _ => unreachable!(),
        }
    }
}

// Length proportional to radius of a cubic bezier handle for 90deg arcs.
const KAPPA90: f32 = 0.552_284_8; // 0.5522847493

pub struct Picture {
    pub commands: Vec<f32>,
    pub cmd: Point,
    pub xform: Transform,
}

impl Picture {
    pub(crate) fn append_commands(&mut self, vals: &mut [f32]) {
        if vals[0] as u32 != CLOSE && vals[0] as u32 != WINDING {
            self.cmd.x = vals[vals.len()-2];
            self.cmd.y = vals[vals.len()-1];
        }
        transform_commands(vals, &self.xform);
        self.commands.extend_from_slice(vals);
    }

    pub fn close_path(&mut self) {
        self.commands.push(CLOSE as f32);
    }

    pub fn path_winding(&mut self, dir: Winding) {
        let dir = dir as i32 as f32;
        self.commands.extend_from_slice(&[ WINDING as f32, dir ]);
    }

    pub fn move_to(&mut self, p: Point) {
        let Point { x, y, .. } = self.xform.transform_point(p);
        self.commands.extend_from_slice(&[ MOVETO as f32, x, y ]);
    }

    pub fn line_to(&mut self, p: Point) {
        let Point { x, y, .. } = self.xform.transform_point(p);
        self.commands.extend_from_slice(&[ LINETO as f32, x, y ]);
    }

    pub fn bezier_to(&mut self, p1: Point, p2: Point, p3: Point) {
        let Point { x: x1, y: y1, .. } = self.xform.transform_point(p1);
        let Point { x: x2, y: y2, .. } = self.xform.transform_point(p2);
        let Point { x: x3, y: y3, .. } = self.xform.transform_point(p3);
        self.commands.extend_from_slice(&[ BEZIERTO as f32, x1, y1, x2, y2, x3, y3 ]);
    }

    pub fn quad_to(&mut self, c: Point, p1: Point) {
        const FIX: f32 = 2.0/3.0;
        let p0 = self.cmd;
        self.bezier_to(p0 + (c - p0) * FIX, p1 + (c - p1) * FIX, p1);
    }

    pub fn rect(&mut self, r: Rect) {
        self.append_commands(&mut [
            MOVETO as f32, r.min.x, r.min.y,
            LINETO as f32, r.min.x, r.max.y,
            LINETO as f32, r.max.x, r.max.y,
            LINETO as f32, r.max.x, r.min.y,
            CLOSE as f32
        ]);
    }

    pub fn rrect_varying(
        &mut self,
        rect: Rect,
        tl: f32,
        tr: f32,
        br: f32,
        bl: f32,
    ) {
        let [x, y, w, h] = rect.to_xywh();
        if tl < 0.1 && tr < 0.1 && br < 0.1 && bl < 0.1 {
            self.rect(rect);
        } else {
            let halfw = w.abs()*0.5;
            let halfh = h.abs()*0.5;
            let sign = if w < 0.0 { -1.0 } else { 1.0 };
            let rx_bl = sign * halfw.min(bl);
            let ry_bl = sign * halfh.min(bl);
            let rx_br = sign * halfw.min(br);
            let ry_br = sign * halfh.min(br);
            let rx_tr = sign * halfw.min(tr);
            let ry_tr = sign * halfh.min(tr);
            let rx_tl = sign * halfw.min(tl);
            let ry_tl = sign * halfh.min(tl);
            let kappa = 1.0 - KAPPA90;
            self.append_commands(&mut [
                MOVETO as f32, x, y + ry_tl,
                LINETO as f32, x, y + h - ry_bl,
                BEZIERTO as f32,
                x, y + h - ry_bl*kappa, x + rx_bl*kappa, y + h, x + rx_bl, y + h,
                LINETO as f32, x + w - rx_br, y + h,
                BEZIERTO as f32,
                x + w - rx_br*kappa, y + h, x + w, y + h - ry_br*kappa, x + w, y + h - ry_br,
                LINETO as f32, x + w, y + ry_tr,
                BEZIERTO as f32,
                x + w, y + ry_tr*kappa, x + w - rx_tr*kappa, y, x + w - rx_tr, y,
                LINETO as f32, x + rx_tl, y,
                BEZIERTO as f32,
                x + rx_tl*kappa, y, x, y + ry_tl*kappa, x, y + ry_tl,
                CLOSE as f32,
            ]);
        }
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) {
        self.append_commands(&mut [
            MOVETO as f32, cx-rx, cy,
            BEZIERTO as f32, cx-rx, cy+ry*KAPPA90, cx-rx*KAPPA90, cy+ry, cx, cy+ry,
            BEZIERTO as f32, cx+rx*KAPPA90, cy+ry, cx+rx, cy+ry*KAPPA90, cx+rx, cy,
            BEZIERTO as f32, cx+rx, cy-ry*KAPPA90, cx+rx*KAPPA90, cy-ry, cx, cy-ry,
            BEZIERTO as f32, cx-rx*KAPPA90, cy-ry, cx-rx, cy-ry*KAPPA90, cx-rx, cy,
            CLOSE as f32,
        ]);
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32) {
        self.ellipse(cx, cy, r, r);
    }

    pub fn arc(&mut self, c: Point, r: f32, a0: f32, a1: f32, dir: Winding) {
        use std::f32::consts::PI;

        let mov = if !self.commands.is_empty() { LINETO } else { MOVETO };

        // Clamp angles
        let mut da = a1 - a0;
        if dir == Winding::CW {
            if da.abs() >= PI*2.0 {
                da = PI*2.0;
            } else {
                while da < 0.0 {
                    da += PI*2.0;
                }
            }
        } else if da.abs() >= PI*2.0 {
            da = -PI*2.0;
        } else {
            while da > 0.0 {
                da -= PI*2.0;
            }
        }

        // Split arc into max 90 degree segments.
        let ndivs = ((da.abs() / (PI*0.5) + 0.5) as i32).clamp(1, 5);
        let hda = (da / ndivs as f32) / 2.0;
        let kappa = (4.0 / 3.0 * (1.0 - hda.cos()) / hda.sin()).abs();
        let kappa = if dir == Winding::CCW { -kappa } else { kappa };

        let mut vals = [0f32; 3 + 5*7];
        let mut nvals = 0;
        let mut prev = point2(0.0, 0.0);
        let mut prev_tan: Vector = vec2(0.0, 0.0);
        for i in 0..=ndivs {
            let angle = a0 + da * (i as f32 / ndivs as f32);
            let (sn, cs) = angle.sin_cos();
            let v = vec2(cs, sn) * r;
            let point = c + v;
            let tan = vec2(-v.y, v.x) * kappa;

            if i == 0 {
                vals[nvals    ] = mov as f32;
                vals[nvals + 1] = point.x;
                vals[nvals + 2] = point.y;
                nvals += 3;
            } else {
                vals[nvals    ] = BEZIERTO as f32;
                vals[nvals + 1] = prev.x+prev_tan.x;
                vals[nvals + 2] = prev.y+prev_tan.y;
                vals[nvals + 3] = point.x-tan.x;
                vals[nvals + 4] = point.y-tan.y;
                vals[nvals + 5] = point.x;
                vals[nvals + 6] = point.y;
                nvals += 7;
            }
            prev = point;
            prev_tan = tan;
        }

        self.append_commands(&mut vals[..nvals]);
    }


    pub fn arc_to(&mut self, p1: Point, p2: Point, radius: f32, dist_tol: f32) {
        let p0 = self.cmd;
        let tol = point2(dist_tol, dist_tol);
        let tol2 = dist_tol * dist_tol;

        if self.commands.is_empty() {
            return;
        }

        // Handle degenerate cases.
        if radius < dist_tol || p0.approx_eq_eps(&p1, &tol) || p2.approx_eq_eps(&p2, &tol) ||
            dist_pt_seg(p1.to_vector(), p0.to_vector(), p2.to_vector()) < tol2 {
            self.line_to(p1);
            return;
        }

        // Calculate tangential circle to lines (x0,y0)-(x1,y1) and (x1,y1)-(x2,y2).
        let d0 = (p0 - p1).normalize();
        let d1 = (p2 - p1).normalize();
        let a = d0.dot(d1).acos();
        let d = radius / (a/2.0).tan();

        //printf("a=%f° d=%f\n", a/NVG_PI*180.0f, d);

        if d > 10_000.0 {
            self.line_to(p1);
            return;
        }

        let (cx, cy, a0, a1, dir);
        if d0.cross(d1) > 0.0 {
            cx = p1.x + d0.x*d + d0.y*radius;
            cy = p1.y + d0.x*d - d0.y*radius;
            a0 = ( d0.x).atan2(-d0.y);
            a1 = (-d1.x).atan2( d1.y);
            dir = Winding::CW;
        } else {
            cx = p1.x + d0.x*d - d0.y*radius;
            cy = p1.y + d0.y*d + d0.x*radius;
            a0 = (-d0.x).atan2( d0.y);
            a1 = ( d1.x).atan2(-d1.y);
            dir = Winding::CCW;
        }

        self.arc(point2(cx, cy), radius, a0, a1, dir);
    }
}