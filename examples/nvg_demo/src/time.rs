use std::time::{Duration, Instant};

pub struct Counter {
    time: Instant,
    tt: Duration,
    deltas: [f32; 240],
    pub index: usize,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            time: Instant::now(),
            tt: Duration::new(1, 0),
            deltas: [0.0f32; 240],
            index: 0,
        }
    }

    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        let delta = now.duration_since(self.time);
        self.time = now;
        self.tt += delta;
        let time = self.tt.as_secs_f32();

        self.deltas[self.index] = delta.as_secs_f32();
        self.index += 1;
        self.index %= self.deltas.len();

        time
    }

    pub fn average_ms(&self) -> f32 {
        let sum: f32 = self.deltas.iter().sum();
        sum / self.deltas.len() as f32 * 1000.0
    }
}
