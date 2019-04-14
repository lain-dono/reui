use std::fmt::Write;
use std::time::{Instant, Duration};

use arrayvec::ArrayString;

use crate::vg::*;
use crate::vg::utils::min;
use crate::context::{Context, Align};

#[repr(i32)]
pub enum GraphStyle {
    Fps = 0,
    Ms = 1,
    Percent = 2,
}

const GRAPH_HISTORY_COUNT: usize = 100;

struct Ticker(Instant);

impl Ticker {
    fn new() -> Self {
        Ticker(Instant::now())
    }

    fn tick(&mut self) -> Duration {
        let old = std::mem::replace(&mut self.0, Instant::now());
        self.0.duration_since(old)
    }

    fn tick_f32(&mut self) -> f32 {
        let dt = self.tick();
        dt.as_secs() as f32 + (dt.subsec_nanos() as f32 / 1.0e9)
    }

    fn tick_f64(&mut self) -> f64 {
        let dt = self.tick();
        dt.as_secs() as f64 + (dt.subsec_nanos() as f64 / 1.0e9)
    }
}

#[repr(C)]
pub struct PerfGraph {
    style: GraphStyle,
    head: usize,
    values: [f32; GRAPH_HISTORY_COUNT],
    name: String,
}

#[no_mangle] extern "C"
fn initGraph(fps: &mut PerfGraph, style: GraphStyle, name: *const i8) {
    let name = unsafe { std::ffi::CStr::from_ptr(name).to_string_lossy() };
    *fps = PerfGraph::new(style, &name);
}
#[no_mangle] extern "C"
fn updateGraph(fps: &mut PerfGraph, dt: f32) {
    fps.update(dt);
}
#[no_mangle] extern "C"
fn renderGraph(vg: &mut Context, x: f32, y: f32, fps: &PerfGraph) {
    fps.render(vg, x, y);
}
#[no_mangle] extern "C"
fn getGraphAverage(fps: &PerfGraph) -> f32 {
    fps.average()
}

impl PerfGraph {
    pub fn new(style: GraphStyle, name: &str) -> Self {
        Self {
            name: name.to_string(),
            style,
            values: [0f32; GRAPH_HISTORY_COUNT],
            head: 0,
            //now: Instant::now(),
        }
    }

    pub fn set_style(&mut self, style: GraphStyle) {
        self.style = style;
    }

    pub fn update(&mut self, frame_time: f32) {
        self.head = (self.head + 1) % GRAPH_HISTORY_COUNT;
        self.values[self.head] = frame_time;
    }

    pub fn average(&self) -> f32 {
        self.values.iter().fold(0.0, |a, v| a + v) / GRAPH_HISTORY_COUNT as f32
    }

    fn get(&self, idx: usize) -> f32 {
        self.values[(self.head+idx) % self.values.len()]
    }

    pub fn render(&self, vg: &mut Context, x: f32, y: f32) {
        let avg = self.average();
        let w = 200.0;
        let h = 35.0;

        vg.begin_path();
        vg.rect(x,y, w,h);
        vg.fill_color(Color::new(0x80_000000));
        vg.fill();

        vg.begin_path();
        vg.move_to(x, y+h);

        match self.style {
            GraphStyle::Fps => for i in 0..self.values.len() {
                let v = min(1.0 / (0.00001 + self.get(i)), 80.0);
                let vx = x + (i as f32 / (GRAPH_HISTORY_COUNT-1) as f32) * w;
                let vy = y + h - ((v / 80.0) * h);
                vg.line_to(vx, vy);
            },
            GraphStyle::Percent => for i in 0..self.values.len() {
                let v = min(self.get(i) * 1.0, 100.0);
                let vx = x + (i as f32 / (GRAPH_HISTORY_COUNT-1) as f32) * w;
                let vy = y + h - ((v / 100.0) * h);
                vg.line_to(vx, vy);
            },
            GraphStyle::Ms => for i in 0..self.values.len() {
                let v = min(self.get(i) * 1000.0, 20.0);
                let vx = x + (i as f32 / (GRAPH_HISTORY_COUNT-1) as f32) * w;
                let vy = y + h - ((v / 20.0) * h);
                vg.line_to(vx, vy);
            },
        }

        vg.line_to(x+w, y+h);
        vg.fill_color(Color::new(0x80_FFC000));
        vg.fill();

        vg.font_face(b"sans\0");

        vg.font_size(14.0);
        vg.text_align(Align::LEFT|Align::TOP);
        vg.fill_color(Color::new(0xC0_F0F0F0));
        vg.text(x+3.0,y+1.0, &self.name);

        let mut s = ArrayString::<[_; 16]>::new();

        vg.font_size(18.0);
        vg.text_align(Align::RIGHT|Align::TOP);
        vg.fill_color(Color::new(0xFF_F0F0F0));
        match self.style {
            GraphStyle::Fps => {
                let _ = s.write_fmt(format_args!("{:.2} FPS", 1.0 / avg));
                vg.text(x+w-3.0,y+1.0, &s);

                s.clear();

                vg.font_size(15.0);
                vg.text_align(Align::RIGHT|Align::BOTTOM);
                vg.fill_color(Color::new(0xA0_f0f0f0));
                let _ = s.write_fmt(format_args!("{:.2} ms", avg * 1000.0));
                vg.text(x+w-3.0,y+h-1.0, &s);
            },
            GraphStyle::Percent => {
                let _ = s.write_fmt(format_args!("{:.1} %", avg * 1.0));
                vg.text(x+w-3.0,y+1.0, &s);
            },
            GraphStyle::Ms => {
                let _ = s.write_fmt(format_args!("{:.2} ms", avg * 1000.0));
                vg.text(x+w-3.0,y+1.0, &s);
            },
        }
    }
}
