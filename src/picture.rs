use crate::{
    Transform,
    Point,
    transform_pt,
};

pub struct Picture {
    pub commands: Vec<f32>,
    pub cmd: Point,
    pub xform: Transform,
}

impl Picture {
    pub(crate) fn append_commands(&mut self, vals: &mut [f32]) {
        use crate::draw_api::{MOVETO, LINETO, BEZIERTO, CLOSE, WINDING};

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
}