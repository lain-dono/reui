struct Point { x: f32, y: f32 }
fn point(x: f32, y: f32) -> Point {
    Point { x, y }
}


const LINE: u32 = 4;
const QUAD: u32 = 6;
const CUBIC: u32 = 8;
const RAT_QUAD: u32 = 7;
const RAT_CUBIC: u32 = 10;

#[repr(C)]
struct Rem {
    line: u32,
    quad: u32,
    cubic: u32,
    rat_quad: u32,
    rat_cubic: u32,
}

impl Rem {
    fn as_array(&mut self) -> &mut [u32; 5] {
        unsafe { std::mem::transmute(self) }
    }
}

pub trait PathBuilderImpl {
    type Result;

    fn begin(&mut self) -> Self::Result;
    fn end(&mut self, path: &mut Path) -> Self::Result;
    fn release(&mut self) -> Self::Result;
    fn flush(&mut self) -> Self::Result;
}

pub struct PathBuilder {
    // impl ptr
    // + impl line/quad/etc

    coords_line: [*mut f32; LINE],
    coords_quad: [*mut f32; QUAD],
    coords_cubic: [*mut f32; CUBIC],
    coords_rat_quad: [*mut f32; RAT_QUAD],
    coords_rat_cubic: [*mut f32; RAT_CUBIC],

    curr: [Point; 2],
}

impl PathBuilder {
    pub fn move_to(&mut self, x0: f32, y0: f32) {
        self.move_to_1(x0, y0);
    }

    pub fn line_to(&mut self, x1: f32, y1: f32) {
        self.acquire(LINE);
        self.coords_append(LINE, 0, self.curr[0].x);
        self.coords_append(LINE, 1, self.curr[0].y);
        self.coords_append(LINE, 2, x1);
        self.coords_append(LINE, 3, y1);
        self.move_to_1(x1, y1);
    }

    pub fn quad_to(
        &mut self,
        x1: f32, y1: f32,
        x2: f32, y2: f32,
    ) {
        self.acquire(QUAD);
        self.coords_append(QUAD, 0, self.curr[0].x);
        self.coords_append(QUAD, 1, self.curr[0].y);
        self.coords_append(QUAD, 2, x1);
        self.coords_append(QUAD, 3, y1);
        self.coords_append(QUAD, 4, x2);
        self.coords_append(QUAD, 5, y2);
        self.move_to_2(x2, y2, x1, y1)
    }
    pub fn quad_smooth_to(
        &mut self,
        x2: f32, y2: f32,
    ) {
        let x1 = self.curr[0].x * 2.0 - self.curr[1].x;
        let y1 = self.curr[0].y * 2.0 - self.curr[1].y;
        self.quad_to(x1, y1, x2, y2)
    }

    pub fn cubic_to(
        &mut self,
        x1: f32, y1: f32,
        x2: f32, y2: f32,
        x3: f32, y3: f32,
    ) {
        self.acquire(CUBIC);
        self.coords_append(CUBIC, 0, self.curr[0].x);
        self.coords_append(CUBIC, 1, self.curr[0].y);
        self.coords_append(CUBIC, 2, x1);
        self.coords_append(CUBIC, 3, y1);
        self.coords_append(CUBIC, 4, x2);
        self.coords_append(CUBIC, 5, y2);
        self.coords_append(CUBIC, 6, x3);
        self.coords_append(CUBIC, 7, y3);
        self.move_to_2(x3, y3, x2, y2);
    }
    pub fn cubic_smooth_to(
        &mut self,
        x2: f32, y2: f32,
        x3: f32, y3: f32,
    ) {
        let x1 = self.curr[0].x * 2.0 - self.curr[1].x;
        let y1 = self.curr[0].y * 2.0 - self.curr[1].y;
        self.cubic_to(x1, y1, x2, y2, x3, y3)
    }

    pub fn rat_quad_to(
        &mut self,
        x1: f32, y1: f32,
        x2: f32, y2: f32,
        w0: f32,
    ) {
        self.acquire(RAT_QUAD);
        self.coords_append(RAT_QUAD, 0, self.curr[0].x);
        self.coords_append(RAT_QUAD, 1, self.curr[0].y);
        self.coords_append(RAT_QUAD, 2, x1);
        self.coords_append(RAT_QUAD, 3, y1);
        self.coords_append(RAT_QUAD, 4, x2);
        self.coords_append(RAT_QUAD, 5, y2);
        self.coords_append(RAT_QUAD, 6, w0);
        self.move_to_1(x2, y2);
    }
    pub fn rat_cubic_to(
        &mut self,
        x1: f32, y1: f32,
        x2: f32, y2: f32,
        x3: f32, y3: f32,
        w0: f32, w1: f32,
    ) {
        self.acquire(RAT_CUBIC);
        self.coords_append(RAT_CUBIC, 0, self.curr[0].x);
        self.coords_append(RAT_CUBIC, 1, self.curr[0].y);
        self.coords_append(RAT_CUBIC, 2, x1);
        self.coords_append(RAT_CUBIC, 3, y1);
        self.coords_append(RAT_CUBIC, 4, x2);
        self.coords_append(RAT_CUBIC, 5, y2);
        self.coords_append(RAT_CUBIC, 6, x3);
        self.coords_append(RAT_CUBIC, 7, y3);
        self.coords_append(RAT_CUBIC, 8, w0);
        self.coords_append(RAT_CUBIC, 9, w1);
        self.move_to_1(x3,y3);
    }
}

impl PathBuilder {
    fn move_to_1(&mut self, x0: f32, y0: f32) {
        self.curr[0] = point(x0, y0);
        self.curr[1] = point(x0, y0);
    }

    fn move_to_2(&mut self, x0: f32, y0: f32, x1: f32, y1: f32) {
        self.curr[0] = point(x0, y0);
        self.curr[1] = point(x1, y1);
    }
}

impl PathBuilder {
    pub fn ellipse(&mut self, cx: f32, cy: f32, rx: f32, ry: f32) {
        //
        // FIXME -- we can implement this with rationals later...
        //
        //
        // Approximate a circle with 4 cubics:
        //
        // http://en.wikipedia.org/wiki/B%C3%A9zier_spline#Approximating_circular_arcs
        //
        self.move_to_1(cx, cy + ry);
        const KAPPA: f32 = 0.55228474983079339840; // moar digits!
        let kx = rx * KAPPA;
        let ky = ry * KAPPA;
        self.cubic_to(
            cx + kx, cy + ry,
            cx + rx, cy + ky,
            cx + rx, cy,
        );
        self.cubic_to(
            cx + rx, cy - ky,
            cx + kx, cy - ry,
            cx,      cy - ry,
        );
        self.cubic_to(
            cx - kx, cy - ry,
            cx - rx, cy - ky,
            cx - rx, cy,
        );
        self.cubic_to(
            cx - rx, cy + ky,
            cx - kx, cy + ry,
            cx,      cy + ry,
        );
    }
}

fn rat_curve_point(t: f32, a: [f32; 2], b: [f32; 2], c: [f32; 2], w1: f32, w2: f32, w3: f32) -> [f32; 2] {
	let s = 1.0 - t;
	let s2 = s * s;
	let t2 = t * t;

    let w1s2 = w1*s2;
    let w2st = w2*2.0*s*t;
    let w3t2 = w3*t2;

    let [ax, ay] = a;
    let [bx, by] = b;
    let [cx, cy] = c;

    [
        (ax*w1s2 + bx*w2st + cx*w3t2) / (w1s2 + w2st + w3t2),
        (ay*w1s2 + by*w2st + cy*w3t2) / (w1s2 + w2st + w3t2),
    ]
}
