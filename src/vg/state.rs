use crate::{
    cache::{LineCap, LineJoin},
    vg::*,
    font::Align,
    math::{rect, Rect, Transform},
};

#[derive(Clone)]
pub struct State {
    pub shape_aa: bool,

    pub fill: Paint,
    pub stroke: Paint,

    pub stroke_width: f32,
    pub miter_limit: f32,
    pub line_join: LineJoin,
    pub line_cap: LineCap,
    pub alpha: f32,
    pub xform: Transform,
    pub scissor: Scissor,

    pub font_size: f32,
    pub letter_spacing: f32,
    pub line_height: f32,
    pub font_blur: f32,
    pub text_align: Align,
    pub font_id: i32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            fill: Paint::color(Color::new(0xFF_FFFFFF)),
            stroke: Paint::color(Color::new(0xFF_000000)),

            shape_aa: true,
            stroke_width: 1.0,
            miter_limit: 10.0,
            line_cap: LineCap::Butt,
            line_join: LineJoin::Miter,
            alpha: 1.0,
            xform: Transform::identity(),

            scissor: Scissor {
                extent: [-1.0, -1.0],
                xform: Transform::identity(),
            },

            font_size: 16.0,
            letter_spacing: 0.0,
            line_height: 1.0,
            font_blur: 0.0,
            text_align: Align::LEFT | Align::BASELINE,
            font_id: 0,
        }
    }
}

impl State {
    pub fn set_scissor(&mut self, rect: Rect) {
        let (x, y) = rect.origin.into();
        let w = rect.size.width.max(0.0);
        let h = rect.size.height.max(0.0);

        self.scissor.xform = Transform::create_translation(x+w*0.5, y+h*0.5)
            .post_transform(&self.xform);

        self.scissor.extent[0] = w*0.5;
        self.scissor.extent[1] = h*0.5;
    }

    pub fn intersect_scissor(&mut self, r: Rect) {
        // If no previous scissor has been set, set the scissor as current scissor.
        if self.scissor.extent[0] < 0.0 {
            self.set_scissor(r);
            return;
        }

        // Transform the current scissor rect into current transform space.
        // If there is difference in rotation, this will be approximation.
        let inv = self.xform.inverse().unwrap_or_else(Transform::identity);
        let xform = self.scissor.xform.post_transform(&inv);

        let ex = self.scissor.extent[0];
        let ey = self.scissor.extent[1];
        let tex = ex*xform.m11.abs() + ey*xform.m21.abs();
        let tey = ex*xform.m12.abs() + ey*xform.m22.abs();

        // Intersect rects.
        let (ax, ay) = (r.origin.x, r.origin.y);
        let (aw, ah) = (r.size.width, r.size.height);

        let (bx, by) = (xform.m31-tex,xform.m32-tey);
        let (bw, bh) = (tex*2.0,tey*2.0);

        //let r1 = rect(bx, by, bw, bh);
        //self.set_scissor(r1.intersection(&r).unwrap_or_else(Rect::zero));

        let minx = f32::max(ax, bx);
        let miny = f32::max(ay, by);
        let maxx = f32::min(ax+aw, bx+bw);
        let maxy = f32::min(ay+ah, by+bh);
        self.set_scissor(rect(
            minx, miny,
            (maxx - minx).max(0.0),
            (maxy - miny).max(0.0),
        ));
    }

    pub fn reset_scissor(&mut self) {
        self.scissor.xform = Transform::identity();
        self.scissor.extent = [-1.0, -1.0];
    }
}