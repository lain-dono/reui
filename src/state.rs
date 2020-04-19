use crate::{
    backend::Scissor,
    math::{point2, Rect, Transform},
};

#[derive(Clone)]
pub struct State {
    pub xform: Transform,
    pub scissor: Scissor,
}

impl Default for State {
    fn default() -> Self {
        Self {
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

        let translate = Transform::translation(x + w * 0.5, y + h * 0.5);
        self.scissor.xform = self.xform.append(translate);

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
