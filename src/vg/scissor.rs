use crate::context::Context;
use crate::transform::{self, Transform};
use super::utils::{minf, maxf};

#[derive(Clone)]
pub struct Scissor {
    pub xform: Transform,
    pub extent: [f32; 2],
}

impl Context {
    pub fn scissor(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let state = self.states.last_mut();

        let w = maxf(0.0, w);
        let h = maxf(0.0, h);

        state.scissor.xform = Transform::create_translation(
            x+w*0.5,
            y+h*0.5,
        ).post_mul(&state.xform);

        state.scissor.extent[0] = w*0.5;
        state.scissor.extent[1] = h*0.5;
    }

    pub fn intersect_scissor(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let state = self.states.last_mut();

        // If no previous scissor has been set, set the scissor as current scissor.
        if state.scissor.extent[0] < 0.0 {
            self.scissor(x, y, w, h);
            return;
        }

        // Transform the current scissor rect into current transform space.
        // If there is difference in rotation, this will be approximation.
        let pxform = state.scissor.xform.pre_mul(
            &state.xform.inverse().unwrap_or_else(Transform::identity));

        let ex = state.scissor.extent[0];
        let ey = state.scissor.extent[1];
        let tex = ex*pxform.m11.abs() + ey*pxform.m21.abs();
        let tey = ex*pxform.m12.abs() + ey*pxform.m22.abs();

        // Intersect rects.
        let rect = isect_rects([pxform.m31-tex,pxform.m32-tey,tex*2.0,tey*2.0], [x,y,w,h]);

        self.scissor(rect[0], rect[1], rect[2], rect[3]);
    }

    pub fn reset_scissor(&mut self) {
        let state = self.states.last_mut();
        state.scissor.xform = Transform::identity();
        state.scissor.extent = [-1.0, -1.0];
    }
}

fn isect_rects(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let [ax, ay, aw, ah] = a;
    let [bx, by, bw, bh] = b;

    let minx = maxf(ax, bx);
    let miny = maxf(ay, by);
    let maxx = minf(ax+aw, bx+bw);
    let maxy = minf(ay+ah, by+bh);
    [
        minx,
        miny,
        maxf(0.0, maxx - minx),
        maxf(0.0, maxy - miny),
    ]
}
