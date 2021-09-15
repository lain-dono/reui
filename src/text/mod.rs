mod font;
mod font_db;
mod paragraph;
mod shaper;
mod style;

pub use self::{
    font::Font,
    font_db::{Database as FontDatabase, FaceId, Family as FontFamily, Query},
    paragraph::Paragraph,
    style::{
        Stretch as FontStretch, Style as FontStyle, TextAnchor, TextDecoration,
        TextDecorationStyle, TextStyle, Weight as FontWeight,
    },
};

impl ttf_parser::OutlineBuilder for crate::Path {
    fn move_to(&mut self, x: f32, y: f32) {
        self.move_to(crate::Offset::new(x, -y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.line_to(crate::Offset::new(x, -y));
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.quad_to(crate::Offset::new(x1, -y1), crate::Offset::new(x, -y));
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.cubic_to(
            crate::Offset::new(x1, -y1),
            crate::Offset::new(x2, -y2),
            crate::Offset::new(x, -y),
        );
    }
    fn close(&mut self) {
        crate::Path::close(self);
    }
}
