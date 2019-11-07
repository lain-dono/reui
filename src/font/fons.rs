use crate::vg::State;

use super::stash::{
    Stash,
    TextIter,
    fonsTextIterInit,
};

pub const FONS_INVALID: i32 = -1;

//const SCRATCH_BUF_SIZE: usize = 96000;
//const HASH_LUT_SIZE: usize = 256;
//const INIT_FONTS: usize = 4;
//const INIT_GLYPHS: usize = 256;
//const INIT_ATLAS_NODES: usize = 256;
//const MAX_FALLBACKS: usize = 20;

pub const GLYPH_BITMAP_OPTIONAL: i32 = 1;
pub const GLYPH_BITMAP_REQUIRED: i32 = 2;

impl Stash {
    pub fn font_by_name(&mut self, name: &str) -> i32 {
        for i in 0..self.fonts.len() {
            let font = &self.fonts[i];
            if name == &font.name {
                return i as i32;
            }
        }
        -1
    }

    // Pull texture changes
    pub fn texture_data(&mut self) -> (i32, i32, *const u8) {
        (self.width, self.height, self.tex_data)
    }

    pub fn validate_texture(&mut self) -> Option<[i32; 4]> {
        let dr = self.dirty_rect;
        if dr[0] < dr[2] && dr[1] < dr[3] {
            // Reset dirty rect
            self.dirty_rect = [
                self.width,
                self.height,
                0,
                0,
            ];
            Some(dr)
        } else {
            None
        }
    }

    pub fn sync_state(&mut self, state: &State, scale: f32) {
        let s = self.state_mut();
        s.size = state.font_size*scale;
        s.spacing = state.letter_spacing*scale;
        s.blur = state.font_blur*scale;
        s.align = state.text_align.bits();
        s.font = state.font_id;
    }

    pub fn text_iter_optional(&mut self,
        x: f32, y: f32,
        start: *const u8, end: *const u8,
    ) -> TextIter {
        let mut iter: TextIter = unsafe { std::mem::zeroed() };
        unsafe {
            fonsTextIterInit(self, &mut iter, x, y,
                start, end,
                GLYPH_BITMAP_OPTIONAL);
            iter.fs = self;
        }
        iter
    }
    
    pub fn text_iter_required(&mut self,
        x: f32, y: f32,
        start: *const u8, end: *const u8,
    ) -> TextIter {
        let mut iter: TextIter = unsafe { std::mem::zeroed() };
        unsafe {
            fonsTextIterInit(self, &mut iter, x, y,
                start, end,
                GLYPH_BITMAP_REQUIRED);
            iter.fs = self;
        }
        iter
    }
}
