use std::{
    mem::size_of,
    cmp::max,
};

extern "C" {
    fn malloc(_: u64) -> *mut libc::c_void;
    fn realloc(_: *mut libc::c_void, _: u64) -> *mut libc::c_void;

    fn memset(_: *mut libc::c_void, _: i32, _: u64) -> *mut libc::c_void;
}

/// Atlas based on Skyline Bin Packer by Jukka Jylänki
#[derive(Clone)]
pub struct Atlas {
    pub width: i16,
    pub height: i16,

    pub nodes: *mut AtlasNode,
    pub nnodes: usize,
    pub cnodes: usize,
}

#[derive(Copy, Clone)]
pub struct AtlasNode {
    pub x: i16,
    pub y: i16,
    pub width: i16,
}

impl Atlas {
    pub fn new(w: i32, h: i32, nnodes: i32) -> Self {
        Self {
            width: w as i16,
            height: h as i16,
            nnodes: 1,
            cnodes: nnodes as usize,

            nodes: unsafe {
                let nodes = malloc((size_of::<AtlasNode>() as u64).wrapping_mul(nnodes as u64)) as *mut AtlasNode;
                assert!(!nodes.is_null());
                memset(nodes as *mut libc::c_void, 0, (size_of::<AtlasNode>() as u64).wrapping_mul(nnodes as u64));

                *nodes = AtlasNode {
                    x: 0,
                    y: 0,
                    width: w as i16,
                };
                nodes
            },
        }
    }

    pub fn reset(&mut self, w: i32, h: i32) {
        self.width = w as i16;
        self.height = h as i16;
        self.nnodes = 1;
        self[0] = AtlasNode {
            x: 0,
            y: 0,
            width: w as i16,
        }
    }

    pub fn add_rect(&mut self, rw: i32, rh: i32) -> Option<(i32, i32)> {
        let (rw, rh) = (rw as i16, rh as i16);

        let mut bestw = self.width;
        let mut besth = self.height;

        let mut besti: i32 = -1;

        let mut bestx = -1;
        let mut besty = -1;

        for i in 0..self.nnodes {
            if let Some(y) = self.rect_fits(i, rw, rh) {
                let node = &self[i];
                if y + rh < besth || y + rh == besth && node.width < bestw {
                    besti = i as i32;
                    bestw = node.width;
                    besth = y + rh;
                    bestx = node.x;
                    besty = y
                }
            }
        }

        if besti == -1 {
            return None;
        }

        if !self.add_skyline_level(besti as usize, bestx, besty, rw, rh) {
            return None;
        }

        Some((i32::from(bestx), i32::from(besty)))
    }

    fn remove_node(&mut self, mut i: usize) {
        if self.nnodes == 0 {
            return;
        }
        while i < self.nnodes - 1 {
            self[i] = self[i + 1];
            i += 1
        }
        self.nnodes -= 1;
    }

    fn insert_node(&mut self, idx: usize, x: i16, y: i16, width: i16) -> bool {
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

        let mut i = self.nnodes;
        self.nnodes += 1;
        while i > idx {
            self[i] = self[i - 1];
            i -= 1
        }
        self[idx] = AtlasNode { x, y, width };
        true
    }

    fn rect_fits(&mut self, mut i: usize, width: i16, height: i16) -> Option<i16> {
        // Checks if there is enough space at the location of skyline span 'i',
        // and return the max height of all skyline spans under that at that location,
        // (think tetris block being dropped at that position). Or -1 if no space found.
        let x = self[i].x;
        let mut y = self[i].y;
        if x + width > self.width {
            return None;
        }
        let mut space_left = width;
        while space_left > 0 {
            if i == self.nnodes {
                return None;
            }
            y = max(y, self[i].y);
            if y + height > self.height {
                return None;
            }
            space_left -= self[i].width;
            i += 1;
        }
        Some(y)
    }

    fn add_skyline_level(
        &mut self,
        idx: usize,
        x: i16,
        y: i16,
        width: i16,
        height: i16,
    ) -> bool {
        if !self.insert_node(idx, x, y + height, width) {
            return false;
        }

        let i = idx + 1;
        while i < self.nnodes {
            if self[i].x >= self[i - 1].x + self[i - 1].width {
                break;
            }

            let shrink = self[i - 1].x + self[i - 1].width - self[i].x;

            self[i].x += shrink;
            self[i].width -= shrink;

            if self[i].width > 0 {
                break;
            }

            self.remove_node(i);
        }

        let mut i = 0;
        while i < self.nnodes - 1 {
            if self[i].y == self[i + 1].y {
                self[i].width += self[i + 1].width;
                self.remove_node(i + 1);
                i -= 1
            }
            i += 1
        }

        true
    }
}

impl std::ops::Index<usize> for Atlas {
    type Output = AtlasNode;
    fn index(&self, index: usize) -> &Self::Output {
        unsafe { &*self.nodes.add(index) }
    }
}

impl std::ops::IndexMut<usize> for Atlas {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        unsafe { &mut *self.nodes.add(index) }
    }
}
