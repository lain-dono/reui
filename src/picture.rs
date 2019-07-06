use crate::{
    Transform,
    Point,
    transform_pt,
    cache::Winding,
    draw_api::{MOVETO, LINETO, BEZIERTO, CLOSE, WINDING},
};

pub struct Picture {
    pub commands: Vec<f32>,
    pub cmd: Point,
    pub xform: Transform,
}

impl Picture {
    pub(crate) fn append_commands(&mut self, vals: &mut [f32]) {
        if vals[0] as i32 != CLOSE && vals[0] as i32 != WINDING {
            self.cmd.x = vals[vals.len()-2];
            self.cmd.y = vals[vals.len()-1];
        }

        // transform commands
        let mut i = 0;
        while i < vals.len() {
            let cmd = vals[i] as i32;
            match cmd {
            MOVETO => {
                transform_pt(&mut vals[i+1..], &self.xform);
                i += 3;
            }
            LINETO => {
                transform_pt(&mut vals[i+1..], &self.xform);
                i += 3;
            }
            BEZIERTO => {
                transform_pt(&mut vals[i+1..], &self.xform);
                transform_pt(&mut vals[i+3..], &self.xform);
                transform_pt(&mut vals[i+5..], &self.xform);
                i += 7;
            }
            CLOSE => i += 1,
            WINDING => i += 2,
            _ => unreachable!(),
            }
        }

        self.commands.extend_from_slice(vals);
    }

    pub fn close_path(&mut self) {
        self.append_commands(&mut [ CLOSE as f32 ]);
    }

    pub fn path_winding(&mut self, dir: Winding) {
        self.append_commands(&mut [ WINDING as f32, dir as i32 as f32 ]);
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.append_commands(&mut [ MOVETO as f32, x, y ]);
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.append_commands(&mut [ LINETO as f32, x, y ]);
    }

    pub fn bezier_to(&mut self, c1x: f32, c1y: f32, c2x: f32, c2y: f32, x: f32, y: f32) {
        self.append_commands(&mut [ BEZIERTO as f32, c1x, c1y, c2x, c2y, x, y ]);
    }
}