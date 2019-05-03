use std::cmp::max;

/// Atlas based on Skyline Bin Packer by Jukka Jylänki
#[derive(Clone)]
pub struct Atlas {
    pub nodes: Vec<AtlasNode>,
    pub width: i16,
    pub height: i16,
}

#[derive(Clone)]
pub struct AtlasNode {
    pub x: i16,
    pub y: i16,
    pub width: i16,
}

impl Atlas {
    pub fn new(width: i32, height: i32, nnodes: i32) -> Self {
        let (width, height) = (width as i16, height as i16);
        let mut nodes = Vec::with_capacity(nnodes as usize);
        nodes.push(AtlasNode { width, x: 0, y: 0 });
        Self { nodes, width, height }
    }

    pub fn reset(&mut self, width: i32, height: i32) {
        let (width, height) = (width as i16, height as i16);
        self.width = width;
        self.height = height;
        self.nodes.clear();
        self.nodes.push(AtlasNode { width, x: 0, y: 0 });
    }

    pub fn add_rect(&mut self, rw: i32, rh: i32) -> Option<(i32, i32)> {
        let (rw, rh) = (rw as i16, rh as i16);

        let mut bestw = self.width;
        let mut besth = self.height;

        let mut besti = None;

        let mut bestx = -1;
        let mut besty = -1;

        for i in 0..self.nodes.len() {
            if let Some(y) = self.rect_fits(i, rw, rh) {
                let node = &self.nodes[i];
                if y + rh < besth || y + rh == besth && node.width < bestw {
                    besti = Some(i);
                    bestw = node.width;
                    besth = y + rh;
                    bestx = node.x;
                    besty = y
                }
            }
        }

        if let Some(idx) = besti {
            self.add_skyline_level(idx, bestx, besty, rw, rh);
            Some((i32::from(bestx), i32::from(besty)))
        } else {
            None
        }
    }

    fn rect_fits(&self, mut i: usize, width: i16, height: i16) -> Option<i16> {
        // Checks if there is enough space at the location of skyline span 'i',
        // and return the max height of all skyline spans under that at that location,
        // (think tetris block being dropped at that position). Or -1 if no space found.
        let x = self.nodes[i].x;
        let mut y = self.nodes[i].y;
        if x + width > self.width {
            return None;
        }
        let mut space_left = width;
        while space_left > 0 {
            if i == self.nodes.len() {
                return None;
            }
            y = max(y, self.nodes[i].y);
            if y + height > self.height {
                return None;
            }
            space_left -= self.nodes[i].width;
            i += 1;
        }
        Some(y)
    }

    fn add_skyline_level(&mut self, idx: usize, x: i16, y: i16, width: i16, height: i16) {
        self.nodes.insert(idx, AtlasNode { x, y: y + height, width });

        let i = idx + 1;
        while i < self.nodes.len() {
            if self.nodes[i].x >= self.nodes[i - 1].x + self.nodes[i - 1].width {
                break;
            }

            let shrink = self.nodes[i - 1].x + self.nodes[i - 1].width - self.nodes[i].x;

            self.nodes[i].x += shrink;
            self.nodes[i].width -= shrink;

            if self.nodes[i].width > 0 {
                break;
            }

            self.nodes.remove(i);
        }

        let mut i = 0;
        while i < self.nodes.len() - 1 {
            if self.nodes[i].y == self.nodes[i + 1].y {
                self.nodes[i].width += self.nodes[i + 1].width;
                self.nodes.remove(i + 1);
                i -= 1
            }
            i += 1
        }
    }
}