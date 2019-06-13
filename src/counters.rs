#[derive(Clone, Copy, Default, Debug)]
pub struct Counters {
    pub draw_call_count: usize,
    pub fill_tri_count: usize,
    pub stroke_tri_count: usize,
    pub text_tri_count: usize,
}

impl Counters {
    pub fn text_call(&mut self, tris: usize) {
        self.text_tri_count += tris;
        self.draw_call_count += 1;
    }

    pub fn stroke_call(&mut self, tris: usize) {
        self.stroke_tri_count += tris;
        self.draw_call_count += 1;
    }

    pub fn fill_call(&mut self, tris: usize) {
        self.fill_tri_count += tris;
        self.draw_call_count += 1;
    }

    pub fn clear(&mut self) {
        self.draw_call_count = 0;
        self.fill_tri_count = 0;
        self.stroke_tri_count = 0;
        self.text_tri_count = 0;
    }
}
