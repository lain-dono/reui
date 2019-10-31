use core::f32::consts::PI;
use crate::{
    context::Context,
    cache::{Winding, LineJoin},
    vg::Color,
    vg::utils::{
        normalize,
        pt_eq,
        dist_pt_seg,
        average_scale,
    },
    Vector,
    Rect,
    rect,
};

use euclid::vec2;

// Length proportional to radius of a cubic bezier handle for 90deg arcs.
const KAPPA90: f32 = 0.552_284_8; // 0.5522847493

pub const MOVETO: i32 = 0;
pub const LINETO: i32 = 1;
pub const BEZIERTO: i32 = 2;
pub const CLOSE: i32 = 3;
pub const WINDING: i32 = 4;

impl Context {
    pub fn fill_rect(&mut self, r: Rect, c: Color) {
        self.begin_path();
        self.rect(r);
        self.fill_color(c);
        self.fill();
    }
}

impl Context {
    pub fn begin_path(&mut self) {
        self.picture.commands.clear();
        self.cache.clear();
    }

    fn append_commands(&mut self, vals: &mut [f32]) {
        self.picture.xform = self.states.last().xform;
        self.picture.append_commands(vals)
    }

    pub fn close_path(&mut self) {
        self.picture.close_path();
    }

    pub fn path_winding(&mut self, dir: Winding) {
        self.picture.path_winding(dir);
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.move_to(x, y);
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.append_commands(&mut [ LINETO as f32, x, y ]);
    }

    pub fn bezier_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.append_commands(&mut [ BEZIERTO as f32, c1x, c1y, c2x, c2y, x, y ]);
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let x0 = self.picture.cmd.x;
        let y0 = self.picture.cmd.y;
        self.append_commands(&mut [
            BEZIERTO as f32,
            x0 + 2.0/3.0*(cx - x0),
            y0 + 2.0/3.0*(cy - y0),
            x  + 2.0/3.0*(cx -  x),
            y  + 2.0/3.0*(cy -  y),
            x, y,
        ]);
    }

    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        let x0 = self.picture.cmd.x;
        let y0 = self.picture.cmd.y;

        if self.picture.commands.is_empty() {
            return;
        }

        // Handle degenerate cases.
        if pt_eq(x0,y0, x1,y1, self.cache.dist_tol) ||
            pt_eq(x1,y1, x2,y2, self.cache.dist_tol) ||
            dist_pt_seg(x1,y1, x0,y0, x2,y2) < self.cache.dist_tol*self.cache.dist_tol ||
            radius < self.cache.dist_tol {
            self.line_to(x1,y1);
            return;
        }

        // Calculate tangential circle to lines (x0,y0)-(x1,y1) and (x1,y1)-(x2,y2).
        let mut dx0 = x0-x1;
        let mut dy0 = y0-y1;
        let mut dx1 = x2-x1;
        let mut dy1 = y2-y1;
        normalize(&mut dx0,&mut dy0);
        normalize(&mut dx1,&mut dy1);
        let a = (dx0*dx1 + dy0*dy1).acos();
        let d = radius / (a/2.0).tan();

        //printf("a=%f° d=%f\n", a/NVG_PI*180.0f, d);

        if d > 10000.0 {
            self.line_to(x1,y1);
            return;
        }

        let (cx, cy, a0, a1, dir);
        let dv: [Vector; 2] = [vec2(dx0,dy0), vec2(dx1,dy1)];
        if dv[0].cross(dv[1]) > 0.0 {
            cx = x1 + dx0*d + dy0*radius;
            cy = y1 + dy0*d + -dx0*radius;
            a0 = ( dx0).atan2(-dy0);
            a1 = (-dx1).atan2( dy1);
            dir = Winding::CW;
            //printf("CW c=(%f, %f) a0=%f° a1=%f°\n", cx, cy, a0/NVG_PI*180.0f, a1/NVG_PI*180.0f);
        } else {
            cx = x1 + dx0*d + -dy0*radius;
            cy = y1 + dy0*d + dx0*radius;
            a0 = (-dx0).atan2( dy0);
            a1 = ( dx1).atan2(-dy1);
            dir = Winding::CCW;
            //printf("CCW c=(%f, %f) a0=%f° a1=%f°\n", cx, cy, a0/NVG_PI*180.0f, a1/NVG_PI*180.0f);
        }

        self.arc(cx, cy, radius, a0, a1, dir);
    }

    pub fn arc(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, dir: Winding) {
        let mov = if !self.picture.commands.is_empty() { LINETO } else { MOVETO };

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
        let (mut px, mut py, mut ptanx, mut ptany) = (0.0, 0.0, 0.0, 0.0);
        for i in 0..=ndivs {
            let a = a0 + da * (i as f32 / ndivs as f32);
            let dx = a.cos();
            let dy = a.sin();
            let x = cx + dx*r;
            let y = cy + dy*r;
            let tanx = -dy*r*kappa;
            let tany = dx*r*kappa;

            if i == 0 {
                vals[nvals    ] = mov as f32;
                vals[nvals + 1] = x;
                vals[nvals + 2] = y;
                nvals += 3;
            } else {
                vals[nvals    ] = BEZIERTO as f32;
                vals[nvals + 1] = px+ptanx;
                vals[nvals + 2] = py+ptany;
                vals[nvals + 3] = x-tanx;
                vals[nvals + 4] = y-tany;
                vals[nvals + 5] = x;
                vals[nvals + 6] = y;
                nvals += 7;
            }
            px = x;
            py = y;
            ptanx = tanx;
            ptany = tany;
        }

        self.append_commands(&mut vals[..nvals]);
    }

    pub fn rect(&mut self, rr: Rect) {
        let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);
        self.append_commands(&mut [
            MOVETO as f32, x,y,
            LINETO as f32, x,y+h,
            LINETO as f32, x+w,y+h,
            LINETO as f32, x+w,y,
            CLOSE as f32
        ]);
    }

    pub fn rrect(&mut self, rr: Rect, radius: f32) {
        self.rrect_varying(rr, radius, radius, radius, radius);
    }

    pub fn rrect_varying(
        &mut self,
        rr: Rect,
        top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32,
    ) {
        let (x, y, w, h) = (rr.origin.x, rr.origin.y, rr.size.width, rr.size.height);
        if top_left < 0.1 && top_right < 0.1 && bottom_right < 0.1 && bottom_left < 0.1 {
            self.rect(rect(x, y, w, h));
        } else {
            let halfw = w.abs()*0.5;
            let halfh = h.abs()*0.5;
            let sign = if w < 0.0 { -1.0 } else { 1.0 };
            let rx_bl = sign * halfw.min(bottom_left );
            let ry_bl = sign * halfh.min(bottom_left );
            let rx_br = sign * halfw.min(bottom_right);
            let ry_br = sign * halfh.min(bottom_right);
            let rx_tr = sign * halfw.min(top_right   );
            let ry_tr = sign * halfh.min(top_right   );
            let rx_tl = sign * halfw.min(top_left    );
            let ry_tl = sign * halfh.min(top_left    );
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

    pub fn fill(&mut self) {
        let state = self.states.last();

        self.cache.flatten_paths(&self.picture.commands);
        self.cache.expand_fill(if state.shape_aa {
            self.cache.fringe_width
        } else {
            0.0
        }, LineJoin::Miter, 2.4);

        // Apply global alpha
        let mut paint = state.fill;
        paint.inner_color.a *= state.alpha;
        paint.outer_color.a *= state.alpha;

        self.params.draw_fill(
            &paint,
            &state.scissor,
            self.cache.fringe_width,
            &self.cache.bounds,
            &self.cache.paths,
        );
    }

    pub fn stroke(&mut self) {
        let state = self.states.last();

        self.run_stroke(
            state.xform,
            state.alpha,
            &Stroke {
                paint: state.stroke,
                scissor: state.scissor,
                width: state.stroke_width,
                line_cap: state.line_cap,
                line_join: state.line_join,
                miter_limit: state.miter_limit,
            },
        );
    }

    pub fn run_stroke(
        &mut self,
        xform: crate::Transform,
        alpha: f32,
        stroke: &Stroke,
    ) {
        let scale = average_scale(&xform);
        let mut stroke_width = (stroke.width * scale).clamp(0.0, 200.0);
        let fringe_width = self.cache.fringe_width;
        let mut paint = stroke.paint;

        if stroke_width < self.cache.fringe_width {
            // If the stroke width is less than pixel size, use alpha to emulate coverage.
            // Since coverage is area, scale by alpha*alpha.
            let alpha = (stroke_width / fringe_width).clamp(0.0, 1.0);
            paint.inner_color.a *= alpha*alpha;
            paint.outer_color.a *= alpha*alpha;
            stroke_width = self.cache.fringe_width;
        }

        // Apply global alpha
        paint.inner_color.a *= alpha;
        paint.outer_color.a *= alpha;

        self.cache.flatten_paths(&self.picture.commands);
        self.cache.expand_stroke(
            stroke_width*0.5,
            fringe_width,
            stroke.line_cap,
            stroke.line_join,
            stroke.miter_limit,
        );

        self.params.draw_stroke(
            &paint,
            crate::vg::CompositeOp::SrcOver.into(),
            &stroke.scissor,
            fringe_width,
            stroke_width,
            &self.cache.paths,
        );
    }
}

pub struct Stroke {
    pub paint: crate::vg::Paint,
    pub scissor: crate::vg::Scissor,
    pub width: f32,
    pub line_cap: crate::cache::LineCap,
    pub line_join: crate::cache::LineJoin,
    pub miter_limit: f32,
}
