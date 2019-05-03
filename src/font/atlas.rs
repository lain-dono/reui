use std::mem::{transmute, size_of};
use std::ptr::null_mut;
use std::cmp::{min, max};

extern "C" {
    fn malloc(_: u64) -> *mut libc::c_void;
    fn realloc(_: *mut libc::c_void, _: u64) -> *mut libc::c_void;
    fn free(__ptr: *mut libc::c_void);

    fn memset(_: *mut libc::c_void, _: i32, _: u64) -> *mut libc::c_void;
}

#[derive(Clone)]
pub struct Atlas {
    pub width: i32,
    pub height: i32,
    pub nodes: *mut AtlasNode,
    pub nnodes: i32,
    pub cnodes: i32,
}

#[derive(Copy, Clone)]
pub struct AtlasNode {
    pub x: i16,
    pub y: i16,
    pub width: i16,
}

impl Atlas {
    fn nodes(&self) -> &[AtlasNode] {
        unsafe { std::slice::from_raw_parts(self.nodes, self.nnodes as usize) }
    }
    fn nodes_mut(&mut self) -> &mut [AtlasNode] {
        unsafe { std::slice::from_raw_parts_mut(self.nodes, self.nnodes as usize) }
    }
 
    pub fn reset(&mut self, w: i32, h: i32) {
        self.width = w;
        self.height = h;
        self.nnodes = 0;
        unsafe {
            (*self.nodes.offset(0)).x = 0;
            (*self.nodes.offset(0)).y = 0;
            (*self.nodes.offset(0)).width = w as i16;
        }
        self.nnodes += 1;
    }

    pub unsafe fn new(w: i32, h: i32, nnodes: i32) -> *mut Atlas {
        let atlas = malloc(size_of::<Atlas>() as u64) as *mut Atlas;
        if !atlas.is_null() {
            memset(
                atlas as *mut libc::c_void,
                0,
                size_of::<Atlas>() as u64,
            );
            (*atlas).width = w;
            (*atlas).height = h;
            (*atlas).nodes =
                malloc((size_of::<AtlasNode>() as u64).wrapping_mul(nnodes as u64))
                    as *mut AtlasNode;
            if !(*atlas).nodes.is_null() {
                memset(
                    (*atlas).nodes as *mut libc::c_void,
                    0,
                    (size_of::<AtlasNode>() as u64).wrapping_mul(nnodes as u64),
                );
                (*atlas).nnodes = 0;
                (*atlas).cnodes = nnodes;
                (*(*atlas).nodes.offset(0)).x = 0;
                (*(*atlas).nodes.offset(0)).y = 0;
                (*(*atlas).nodes.offset(0)).width = w as i16;
                (*atlas).nnodes += 1;
                return atlas;
            }
        }
        if !atlas.is_null() {
            //fons__deleteAtlas(atlas);
        }
        null_mut()
    }

    pub fn add_rect(
        &mut self,
        rw: i32,
        rh: i32,
    ) -> Option<(i32, i32)> {
        let mut bestw: i32 = self.width;
        let mut besth: i32 = self.height;

        let mut besti: i32 = -1;

        let mut bestx: i32 = -1;
        let mut besty: i32 = -1;

        for i in 0..self.nnodes {
            let y = self.rect_fits(i as usize, rw, rh) as i32;
            if y != -1 {
                let node = &self.nodes()[i as usize];
                if y + rh < besth || y + rh == besth && (node.width as i32) < bestw {
                    besti = i;
                    bestw = node.width as i32;
                    besth = y + rh;
                    bestx = node.x as i32;
                    besty = y
                }
            }
        }

        if besti == -1 {
            return None;
        }

        if unsafe { fons__atlasAddSkylineLevel(self, besti, bestx, besty, rw, rh) } == 0 {
            return None;
        }

        Some((bestx, besty))
    }

    pub fn remove_node(&mut self, mut i: usize) {
        if self.nnodes == 0 {
            return;
        }
        while i < self.nnodes as usize - 1 {
            self.nodes_mut()[i] = self.nodes()[i + 1];
            i += 1
        }
        self.nnodes -= 1;
    }

    pub fn insert_node(&mut self, idx: usize, x: i32, y: i32, w: i32) -> bool {
        if self.nnodes + 1 > self.cnodes {
            self.cnodes = if self.cnodes == 0 {
                8
            } else {
                self.cnodes * 2
            };
            let size = (size_of::<AtlasNode>() as u64).wrapping_mul(self.cnodes as u64);
            unsafe {
                self.nodes = realloc(self.nodes as *mut libc::c_void, size) as *mut AtlasNode;
                if self.nodes.is_null() {
                    return false;
                }
            }
        }

        let mut i = self.nnodes as usize;
        self.nnodes += 1;
        while i > idx {
            self.nodes_mut()[i] = self.nodes()[i - 1];
            i -= 1
        }
        self.nodes_mut()[idx] = AtlasNode {
            x: x as i16,
            y: y as i16,
            width: w as i16,
        };
        true
    }

    fn rect_fits(&mut self, mut i: usize, w: i32, h: i32) -> i16 {
        let (w, h) = (w as i16, h as i16);
        // Checks if there is enough space at the location of skyline span 'i',
        // and return the max height of all skyline spans under that at that location,
        // (think tetris block being dropped at that position). Or -1 if no space found.
        let mut x = self.nodes()[i].x;
        let mut y = self.nodes()[i].y;
        if x + w > self.width as i16 {
            return -1;
        }
        let mut space_left = w;
        while space_left > 0 {
            if i == self.nodes().len() {
                return -1;
            }
            y = max(y, self.nodes()[i].y);
            if y + h > self.height as i16 {
                return -1;
            }
            space_left -= self.nodes()[i].width;
            i += 1
        }
        y
    }
}

unsafe fn fons__atlasAddSkylineLevel(
    atlas: *mut Atlas,
    idx: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
) -> i32 {
    if !(*atlas).insert_node(idx as usize, x, y + h, w) {
        return 0;
    }
    let mut i = idx + 1;
    while i < (*atlas).nnodes {
        if (*(*atlas).nodes.offset(i as isize)).x >= (*(*atlas).nodes.offset((i - 1) as isize)).x
                + (*(*atlas).nodes.offset((i - 1) as isize)).width
        {
            break;
        }
        let mut shrink: i32 = (*(*atlas).nodes.offset((i - 1) as isize)).x as i32
            + (*(*atlas).nodes.offset((i - 1) as isize)).width as i32
            - (*(*atlas).nodes.offset(i as isize)).x as i32;
        let ref mut fresh0 = (*(*atlas).nodes.offset(i as isize)).x;
        *fresh0 = (*fresh0 as i32 + shrink as i16 as i32) as i16;
        let ref mut fresh1 = (*(*atlas).nodes.offset(i as isize)).width;
        *fresh1 = (*fresh1 as i32 - shrink as i16 as i32) as i16;
        if !((*(*atlas).nodes.offset(i as isize)).width as i32 <= 0) {
            break;
        }
        (*atlas).remove_node(i as usize);
        i -= 1;
        i += 1
    }
    i = 0;
    while i < (*atlas).nnodes - 1 {
        if (*(*atlas).nodes.offset(i as isize)).y as i32
            == (*(*atlas).nodes.offset((i + 1) as isize)).y as i32
        {
            let ref mut fresh2 = (*(*atlas).nodes.offset(i as isize)).width;
            *fresh2 =
                (*fresh2 as i32 + (*(*atlas).nodes.offset((i + 1) as isize)).width as i32) as i16;
            (*atlas).remove_node(i as usize + 1);
            i -= 1
        }
        i += 1
    }
    1
}