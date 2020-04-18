use crate::{
    canvas::{StrokeCap, StrokeJoin},
    math::{point2, Color, Rect, Transform},
    vg::*,
};

#[derive(Clone)]
pub struct State {
    pub shape_aa: bool,

    pub fill: Paint,
    pub stroke: Paint,

    pub stroke_width: f32,
    pub stroke_miter_limit: f32,
    pub stroke_join: StrokeJoin,
    pub stroke_cap: StrokeCap,

    pub xform: Transform,
    pub scissor: Scissor,
}

impl Default for State {
    fn default() -> Self {
        Self {
            fill: Paint::color(Color::new(0xFF_FFFFFF)),
            stroke: Paint::color(Color::new(0xFF_000000)),

            shape_aa: true,
            stroke_width: 1.0,
            stroke_miter_limit: 10.0,
            stroke_cap: StrokeCap::Butt,
            stroke_join: StrokeJoin::Miter,
            xform: Transform::identity(),

            scissor: Scissor {
                extent: [-1.0, -1.0],
                xform: Transform::identity(),
            },
        }
    }
}

impl State {
    pub fn set_scissor(&mut self, rect: Rect) {
        let [x, y, w, h] = rect.to_xywh();

        self.scissor.xform = self
            .xform
            .append(Transform::translation(x + w * 0.5, y + h * 0.5));

        self.scissor.extent[0] = w * 0.5;
        self.scissor.extent[1] = h * 0.5;
    }

    pub fn intersect_scissor(&mut self, r: Rect) {
        // If no previous scissor has been set, set the scissor as current scissor.
        if self.scissor.extent[0] < 0.0 {
            self.set_scissor(r);
            return;
        }

        // Transform the current scissor rect into current transform space.
        // If there is difference in rotation, this will be approximation.
        let inv = self.xform.inverse();
        let xform = self.scissor.xform.prepend(inv);

        let ex = self.scissor.extent[0];
        let ey = self.scissor.extent[1];

        let tex = ex * xform.re.abs() + ey * xform.im.abs();
        let tey = ex * xform.im.abs() + ey * xform.re.abs();

        // Intersect rects.
        let (ax, ay) = (r.min.x, r.min.y);
        let (aw, ah) = (r.dx(), r.dy());

        let (bx, by) = (xform.tx - tex, xform.ty - tey);
        let (bw, bh) = (tex * 2.0, tey * 2.0);

        let minx = f32::max(ax, bx);
        let miny = f32::max(ay, by);
        let maxx = f32::min(ax + aw, bx + bw);
        let maxy = f32::min(ay + ah, by + bh);
        self.set_scissor(Rect {
            min: point2(minx, miny),
            max: point2(maxx, maxy),
        });
    }

    pub fn reset_scissor(&mut self) {
        self.scissor.xform = Transform::identity();
        self.scissor.extent = [-1.0, -1.0];
    }
}
