// Atlas based on Skyline Bin Packer by Jukka Jylänki

pub struct AtlasNode {
    pub x: i16,
    pub y: i16,
    pub width: i16,
}

pub struct Atlas {
    pub nodes: Vec<AtlasNode>,
    pub width: i32,
    pub height: i32,
}

impl Atlas {
    pub fn new(width: i32, height: i32, capacity: usize) -> Self {
        let mut nodes = Vec::with_capacity(capacity);
        nodes.push(AtlasNode { x: 0, y: 0, width: w as i16 });
        Self { width, height, nodes }
    }

    pub fn insert_node(&mut self, idx: i32, x: i32, y: i32, w: i32) {
        self.nodes.insert(ids as usize, AtlasNode {
            x: x as i16,
            y: y as i16,
            width: w as i16,
        });
    }

    pub fn remove_node(&mut self, idx: i32) {
        self.nodes.remove(idx as usize);
    }

    pub fn expand(&mut self, w: i32, h: i32) {
        // Insert node for empty space
        if w > self.width {
            self.insert_node(self.nnodes, self.width, 0, w - self.width);
        }
        self.width = w;
        self.height = h;
    }

    pub fn reset(&mut self, w: i32, h: i32) {
        self.width = w;
        self.height = h;
        self.nodes.clear();

        // Init root node.
        self.nodes.push(AtlasNode {
            x: 0,
            y: 0,
            width: w as i16,
        });
    }

    pub fn add_skyline_level(&mut self, idx: i32, x: i32, y: i32, w: i32, h: i32) {
        // Insert new node
        self.insert_node(idx, x, y+h, w);

        // Delete skyline segments that fall under the shadow of the new segment.
        for i = idx+1; i < self.nnodes; i++) {
            if self.nodes[i].x >= self.nodes[i-1].x + self.nodes[i-1].width {
                break;
            }

            let shrink = self.nodes[i-1].x + self.nodes[i-1].width - self.nodes[i].x;
            self.nodes[i].x += shrink as i16;
            self.nodes[i].width -= shrink as i16;
            if self.nodes[i].width <= 0 {
                self.remove_node(i);
                i -= 1;
            } else {
                break;
            }
        }

        // Merge same height skyline segments that are next to each other.
        let mut i = 0;
        while i < self.nodes.len()-1 {
            if self.nodes[i].y == self.nodes[i+1].y {
                self.nodes[i].width += self.nodes[i+1].width;
                self.remove_node(i+1);
                i -= 1;
            }
            i += 1;
        }
    }

    pub fn rect_fits(&self, i: i32, w: i32, h: i32) -> i32 {
        // Checks if there is enough space at the location of skyline span 'i',
        // and return the max height of all skyline spans under that at that location,
        // (think tetris block being dropped at that position). Or -1 if no space found.
        let x = self.nodes[i].x as i32;
        let y = self.nodes[i].y as i32;
        if x + w > self.width {
            -1
        } else {
            let mut y = y;
            let mut space_left = w;
            let mut i = 0;
            while space_left > 0 {
                if i == self.nodes.len() { return -1 }
                let y = maxi(y, self.nodes[i].y as i32);
                if y + h > self.height { return -1 }
                space_left -= self.nodes[i].width as i32;
                i += 1;
            }
            y
        }
    }

    pub fn add_rect(&mut self, rw: i32, rh: i32) -> Option<[i32; 2]> {
        let mut best_h = self.height,
        let mut best_w = self.width;
        let mut best_i = -1;
        let mut best_x = -1;
        let mut best_y = -1;

        // Bottom left fit heuristic.
        for node in &self.nodes {
            let y = self.rect_fits(i, rw, rh);
            if y == -1 { break }
            if y + rh < besth || (y + rh == best_h && node.width < best_w) {
                best_i = i;
                best_w = node.width;
                best_h = y + rh;
                best_x = node.x;
                best_y = y;
            }
        }

        // Perform the actual packing.
        if besti == -1 {
            None
        } else {
            self.add_skyline_level(best_i, best_x, best_y, rw, rh);
            Some([best_x, best_y])
        }
    }
}
