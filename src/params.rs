use std::ffi::c_void;
use crate::{
    cache::{Path, Vertex},
    vg::{Scissor, Paint, CompositeState},
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Params {
    pub user_ptr: *mut c_void,
    pub edge_aa: i32,
    create: unsafe extern "C" fn(*mut c_void) -> i32,

    create_texture: unsafe extern "C" fn(
        *mut c_void, _type: i32, w: u32, h: u32, image_flags: i32, data: *const u8) -> u32,
    delete_texture: unsafe extern "C" fn(*mut c_void, u32) -> i32,
    update_texture: unsafe extern "C" fn(*mut c_void, u32, i32, i32, u32, u32, *const u8) -> i32,
    texture_size: unsafe extern "C" fn(*mut c_void, u32, *mut u32, *mut u32) -> i32,

    viewport: unsafe extern "C" fn(*mut c_void, f32, f32, f32),
    cancel: unsafe extern "C" fn(*mut c_void),
    flush: unsafe extern "C" fn(*mut c_void),

    fill: unsafe extern "C" fn(
        *mut c_void, *const Paint, CompositeState, *const Scissor, fringe: f32,
        bounds: *const f32, *const Path, i32),
    stroke: unsafe extern "C" fn(
        *mut c_void, *const Paint, CompositeState, *const Scissor, fringe: f32,
        f32, *const Path, i32),
    triangles: unsafe extern "C" fn(
        *mut c_void, *const Paint, CompositeState, *const Scissor,
        *const Vertex, i32),

    delete: unsafe extern "C" fn(*mut c_void),
}

impl Params {
    pub fn create(&mut self) {
        assert_ne!(unsafe { (self.create)(self.user_ptr) }, 0);
    }

    pub fn delete(&mut self) {
        unsafe { (self.delete)(self.user_ptr); }
    }

    pub fn create_texture(&mut self, _type: i32, w: u32, h: u32, image_flags: i32, data: *const u8) -> u32 {
        unsafe { (self.create_texture)(self.user_ptr, _type, w, h, image_flags, data) }
    }
    pub fn delete_texture(&mut self, image: u32) {
        unsafe { (self.delete_texture)(self.user_ptr, image); }
    }
    pub fn update_texture(&mut self, image: u32, x: i32, y: i32, w: u32, h: u32, data: *const u8) {
        unsafe { (self.update_texture)(self.user_ptr, image, x, y, w, h, data); }
    }

    pub fn texture_size(&mut self, image: u32) -> Option<(u32, u32)> {
        let (mut w, mut h) = (0, 0);
        if unsafe { (self.texture_size)(self.user_ptr, image, &mut w, &mut h) } != 0 {
            Some((w, h))
        } else {
            None
        }
    }

    pub fn viewport(&mut self, w: f32, h: f32, dpi: f32) {
        unsafe { (self.viewport)(self.user_ptr, w, h, dpi); }
    }
    pub fn cancel(&mut self) {
        unsafe { (self.cancel)(self.user_ptr); }
    }
    pub fn flush(&mut self) {
        unsafe { (self.flush)(self.user_ptr); }
    }

    pub fn fill(
        &mut self, paint: &Paint, op: CompositeState, scissor: &Scissor, fringe: f32,
        bounds: &[f32; 4],
        paths: &[Path],
    ) {
        unsafe {
            (self.fill)(self.user_ptr, paint, op, scissor, fringe, bounds.as_ptr(), paths.as_ptr(), paths.len() as i32);
        }
    }
    pub fn stroke(
        &mut self, paint: &Paint, op: CompositeState, scissor: &Scissor, fringe: f32,
        stroke_width: f32, 
        paths: &[Path],
    ) {
        unsafe {
            (self.stroke)(self.user_ptr, paint, op, scissor, fringe, stroke_width, paths.as_ptr(), paths.len() as i32);
        }
    }
    pub fn triangles(&mut self, paint: &Paint, op: CompositeState, scissor: &Scissor, vertex: &[Vertex]) {
        unsafe {
            (self.triangles)(self.user_ptr, paint, op, scissor, vertex.as_ptr(), vertex.len() as i32);
        }
    }
}
