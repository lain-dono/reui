use crate::{
    context::{
        Context,
        States,
    },
    cache::PathCache,
    backend::BackendGL,
    vg::Counters,
    fons::{FONScontext, FONSparams},
};

use std::ptr::null;

pub const TEXTURE_ALPHA: i32 = 0x01;
pub const TEXTURE_RGBA: i32 = 0x02;

const INIT_COMMANDS_SIZE: usize = 256;

const INIT_FONTIMAGE_SIZE: usize = 512;
pub const MAX_FONTIMAGE_SIZE: u32 = 2048;
pub const MAX_FONTIMAGES: usize = 4;

extern "C" {
    fn fonsCreateInternal(params: &FONSparams) -> Box<FONScontext>;
}

impl Context {
    pub fn begin_frame(&mut self, width: f32, height: f32, dpi: f32) {
        log::trace!("draws:{}  fill:{}  stroke:{}  text:{}  TOTAL:{}",
            self.counters.draw_call_count,
            self.counters.fill_tri_count,
            self.counters.stroke_tri_count,
            self.counters.text_tri_count,
            self.counters.fill_tri_count+self.counters.stroke_tri_count+self.counters.text_tri_count,
        );

        self.states.clear();
        self.save();
        self.reset();
        self.set_dpi(dpi);

        self.params.set_viewport(width, height, dpi);

        self.counters.clear();
    }
    pub fn cancel_frame(&mut self) {
        self.params.reset()
    }
    pub fn end_frame(&mut self) {
        self.params.flush();

        if self.font_image_idx == 0 {
            return;
        }

        let font_image = self.font_images[self.font_image_idx as usize];
        // delete images that smaller than current one
        if font_image.0 == 0 {
            return;
        }

        let (iw, ih) = self.image_size(font_image);
        let mut j = 0;
        let font_images = self.font_images;
        for &m in &font_images {
            if m.0 != 0 {
                let (nw, nh) = self.image_size(m);
                if nw < iw || nh < ih {
                    self.delete_image(m);
                } else {
                    self.font_images[j] = m;
                    j += 1;
                }
            }
        }

        // make current font image to first
        self.font_images[j] = self.font_images[0];
        self.font_images[0] = font_image;
        self.font_image_idx = 0;
        j += 1;

        // clear all images after j
        for i in j..MAX_FONTIMAGES {
            self.font_images[i].0 = 0;
        }
    }
}

impl Context {
    pub fn set_dpi(&mut self, ratio: f32) {
        self.cache.set_dpi(ratio);
        self.device_px_ratio = ratio;
    }

    pub fn new(mut params: BackendGL) -> Self {
        let fs_params = FONSparams::simple(INIT_FONTIMAGE_SIZE as i32, INIT_FONTIMAGE_SIZE as i32);
        let fs = unsafe { fonsCreateInternal(&fs_params) };

        let font_image = params.create_texture(
            TEXTURE_ALPHA,
            INIT_FONTIMAGE_SIZE as u32,
            INIT_FONTIMAGE_SIZE as u32,
            Default::default(),
            null(),
        );

        Self {
            params, fs,

            states: States::new(),

            font_images: [
                font_image,
                Default::default(),
                Default::default(),
                Default::default(),
            ],

            commandx: 0.0,
            commandy: 0.0,
            counters: Counters::default(),
            cache: PathCache::new(),
            commands: Vec::with_capacity(INIT_COMMANDS_SIZE),

            //tess_tol: 0.25,
            //dist_tol: 0.01,
            //fringe_width: 1.0,
            device_px_ratio: 1.0,

            font_image_idx: 0,
        }
    }

    pub(crate) fn append_commands(&mut self, vals: &mut [f32]) {
        use crate::draw_api::{MOVETO, LINETO, BEZIERTO, CLOSE, WINDING};
        use crate::transform::transform_pt;

        let xform = &self.states.last().xform;

        if vals[0] != CLOSE as f32 && vals[0] != WINDING as f32 {
            self.commandx = vals[vals.len()-2];
            self.commandy = vals[vals.len()-1];
        }

        // transform commands
        let mut i = 0;
        while i < vals.len() {
            let cmd = vals[i] as i32;
            match cmd {
            MOVETO => {
                transform_pt(&mut vals[i+1..], xform);
                i += 3;
            }
            LINETO => {
                transform_pt(&mut vals[i+1..], xform);
                i += 3;
            }
            BEZIERTO => {
                transform_pt(&mut vals[i+1..], xform);
                transform_pt(&mut vals[i+3..], xform);
                transform_pt(&mut vals[i+5..], xform);
                i += 7;
            }
            CLOSE => i += 1,
            WINDING => i += 2,
            _ => unreachable!(),
            }
        }

        self.commands.extend_from_slice(vals);
    }
}
