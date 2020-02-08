use crate::{
    context::Context,
    cache::{Winding, LineJoin},
    vg::utils::average_scale,
    math::{Transform, Rect, point2},
};

pub struct Stroke {
    pub paint: crate::vg::Paint,
    pub scissor: crate::vg::Scissor,
    pub width: f32,
    pub line_cap: crate::cache::LineCap,
    pub line_join: crate::cache::LineJoin,
    pub miter_limit: f32,
}

impl Context {
    pub fn fill_rect(&mut self, r: Rect, c: u32) {
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

    pub fn close_path(&mut self) {
        self.picture.close_path();
    }

    pub fn path_winding(&mut self, dir: Winding) {
        self.picture.path_winding(dir);
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.move_to(point2(x, y));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.line_to(point2(x, y));
    }

    pub fn bezier_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.bezier_to(point2(c1x, c1y), point2(c2x, c2y), point2(x, y));
    }

    pub fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.quad_to(point2(cx, cy), point2(x, y));
    }

    pub fn arc_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, radius: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.arc_to(point2(x1, y1), point2(x2, y2), radius, self.cache.dist_tol);
    }

    pub fn arc(&mut self, cx: f32, cy: f32, r: f32, a0: f32, a1: f32, dir: Winding) {
        self.picture.xform = self.states.last().xform;
        self.picture.arc(point2(cx, cy), r, a0, a1, dir);
    }

    pub fn rect(&mut self, rect: Rect) {
        self.picture.xform = self.states.last().xform;
        self.picture.rect(rect);
    }

    pub fn rrect(&mut self, rect: Rect, radius: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.rrect_varying(rect, radius, radius, radius, radius);
    }

    pub fn rrect_varying(&mut self, rect: Rect, tl: f32, tr: f32, br: f32, bl: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.rrect_varying(rect, tl, tr, br, bl);
    }

    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.ellipse(cx, cy, rx, ry);
    }

    pub fn circle(&mut self, cx: f32, cy: f32, r: f32) {
        self.picture.xform = self.states.last().xform;
        self.picture.circle(cx, cy, r);
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

        let stroke = Stroke {
            paint: state.stroke,
            scissor: state.scissor,
            width: state.stroke_width,
            line_cap: state.line_cap,
            line_join: state.line_join,
            miter_limit: state.miter_limit,
        };
        let alpha = state.alpha;
        let xform = state.xform;

        self.run_stroke(xform, alpha, &stroke);
    }

    pub fn run_stroke(
        &mut self,
        xform: Transform,
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
            &stroke.scissor,
            fringe_width,
            stroke_width,
            &self.cache.paths,
        );
    }
}