use crate::context::Context;
use crate::transform;
use super::utils::{minf, maxf};

#[repr(C)]
#[derive(Clone)]
pub struct Scissor {
    pub xform: [f32; 6],
    pub extent: [f32; 2],
}

impl Context {
    pub fn scissor(&mut self, x: f32, y: f32, w: f32, h: f32) {
        let state = self.states.last_mut();

        let w = maxf(0.0, w);
        let h = maxf(0.0, h);

        state.scissor.xform = transform::identity();
        state.scissor.xform[4] = x+w*0.5;
        state.scissor.xform[5] = y+h*0.5;
        transform::mul(&mut state.scissor.xform, &state.xform);

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
        let mut pxform = state.scissor.xform;
        let ex = state.scissor.extent[0];
        let ey = state.scissor.extent[1];
        let mut invxorm = [0.0; 6];
        transform::inverse_checked(&mut invxorm, &state.xform);
        transform::mul(&mut pxform, &invxorm);
        let tex = ex*pxform[0].abs() + ey*pxform[2].abs();
        let tey = ex*pxform[1].abs() + ey*pxform[3].abs();

        // Intersect rects.
        let rect = isect_rects([pxform[4]-tex,pxform[5]-tey,tex*2.0,tey*2.0], [x,y,w,h]);

        self.scissor(rect[0], rect[1], rect[2], rect[3]);
    }

    pub fn reset_scissor(&mut self) {
        let state = self.states.last_mut();
        state.scissor.xform = [0.0; 6];
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
