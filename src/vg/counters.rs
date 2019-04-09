#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Counters {
    pub draw_call_count: i32,
    pub fill_tri_count: i32,
    pub stroke_tri_count: i32,
    pub text_tri_count: i32,
}

impl Counters {
    pub fn text_call(&mut self, tris: usize) {
        self.text_tri_count += tris as i32;
        self.draw_call_count += 1;
    }

    pub fn stroke_call(&mut self, tris: usize) {
        self.stroke_tri_count += tris as i32;
        self.draw_call_count += 1;
    }

    pub fn fill_call(&mut self, tris: usize) {
        self.fill_tri_count += tris as i32;
        self.draw_call_count += 1;
    }

    pub fn clear(&mut self) {
        self.draw_call_count = 0;
        self.fill_tri_count = 0;
        self.stroke_tri_count = 0;
        self.text_tri_count = 0;
    }
}
