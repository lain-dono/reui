use core::f32::consts::PI;
use crate::context::Context;
use crate::cache::{Winding, LineJoin};
use crate::vg::utils::{
    clamp,
    min,
    normalize,
    pt_eq,
    dist_pt_seg,
    cross,
    clampf,
    average_scale,
};

// Length proportional to radius of a cubic bezier handle for 90deg arcs.
const KAPPA90: f32 = 0.552_284_8; // 0.5522847493

pub const MOVETO: i32 = 0;
pub const LINETO: i32 = 1;
pub const BEZIERTO: i32 = 2;
pub const CLOSE: i32 = 3;
pub const WINDING: i32 = 4;

impl Context {
    pub fn begin_path(&mut self) {
        self.commands.clear();
        self.cache.clear();
    }

    pub fn close_path(&mut self) {
        self.append_commands(&mut [ CLOSE as f32 ]);
    }

    pub fn path_winding(&mut self, dir: Winding) {
        self.append_commands(&mut [ WINDING as f32, dir as i32 as f32 ]);
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.append_commands(&mut [ MOVETO as f32, x, y ]);
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.append_commands(&mut [ LINETO as f32, x, y ]);
    }

    pub fn bezier_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.append_commands(&mut [ BEZIERTO as f32, c1x, c1y, c2x, c2y, x, y ]);
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        let x0 = self.commandx;
        let y0 = self.commandy;
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
        let x0 = self.commandx;
        let y0 = self.commandy;

        if self.commands.is_empty() {
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
        if cross(dx0,dy0, dx1,dy1) > 0.0 {
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
        let ndivs = clamp((da.abs() / (PI*0.5) + 0.5) as i32, 1, 5);
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

    pub fn rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        self.append_commands(&mut [
            MOVETO as f32, x,y,
            LINETO as f32, x,y+h,
            LINETO as f32, x+w,y+h,
            LINETO as f32, x+w,y,
            CLOSE as f32
        ]);
    }

    pub fn rrect(&mut self, x: f32, y: f32, width: f32, height: f32, r: f32) {
        self.rrect_varying(x, y, width, height, r, r, r, r);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn rrect_varying(
        &mut self,
        x: f32, y: f32, w: f32, h: f32,
        top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32,
    ) {
        if top_left < 0.1 && top_right < 0.1 && bottom_right < 0.1 && bottom_left < 0.1 {
            self.rect(x, y, w, h);
        } else {
            let halfw = w.abs()*0.5;
            let halfh = h.abs()*0.5;
            let sign = if w < 0.0 { -1.0 } else { 1.0 };
            let rx_bl = sign * min(halfw, bottom_left );
            let ry_bl = sign * min(halfh, bottom_left );
            let rx_br = sign * min(halfw, bottom_right);
            let ry_br = sign * min(halfh, bottom_right);
            let rx_tr = sign * min(halfw, top_right   );
            let ry_tr = sign * min(halfh, top_right   );
            let rx_tl = sign * min(halfw, top_left    );
            let ry_tl = sign * min(halfh, top_left    );
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
        let state = self.states.last_mut();

        let w = if self.params.edge_aa() && state.shape_aa != 0 {
            self.cache.fringe_width
        } else {
            0.0
        };

        self.cache.flatten_paths(&self.commands);
        self.cache.expand_fill(w, LineJoin::Miter, 2.4);

        // Apply global alpha
        let mut paint = state.fill;
        paint.inner_color.a *= state.alpha;
        paint.outer_color.a *= state.alpha;

        self.params.draw_fill(
            &paint,
            state.composite,
            &state.scissor,
            self.cache.fringe_width,
            &self.cache.bounds,
            &self.cache.paths,
        );

        // Count triangles
        for path in &self.cache.paths {
            self.counters.fill_call(path.nfill-2);
            self.counters.fill_call(path.nstroke-2);
        }
    }

    pub fn stroke(&mut self) {
        let state = self.states.last_mut();

        let scale = average_scale(&state.xform);
        let mut stroke_width = clampf(state.stroke_width * scale, 0.0, 200.0);
        let mut paint = state.stroke;

        if stroke_width < self.cache.fringe_width {
            // If the stroke width is less than pixel size, use alpha to emulate coverage.
            // Since coverage is area, scale by alpha*alpha.
            let alpha = clampf(stroke_width / self.cache.fringe_width, 0.0, 1.0);
            paint.inner_color.a *= alpha*alpha;
            paint.outer_color.a *= alpha*alpha;
            stroke_width = self.cache.fringe_width;
        }

        // Apply global alpha
        paint.inner_color.a *= state.alpha;
        paint.outer_color.a *= state.alpha;

        let w = if self.params.edge_aa() && state.shape_aa != 0 {
            self.cache.fringe_width
        } else {
            0.0
        };

        self.cache.flatten_paths(&self.commands);
        self.cache.expand_stroke(stroke_width*0.5, w, state.line_cap, state.line_join, state.miter_limit);

        self.params.draw_stroke(
            &paint,
            state.composite,
            &state.scissor,
            self.cache.fringe_width,
            stroke_width,
            &self.cache.paths,
        );

        // Count triangles
        for path in &self.cache.paths {
            self.counters.stroke_call(path.nstroke-2);
        }
    }
}
