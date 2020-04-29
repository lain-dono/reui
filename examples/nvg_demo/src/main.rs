#![warn(clippy::all)]
#![allow(unstable_name_collisions)] // TODO: clamp

mod blendish;
mod canvas;
mod run_wgpu;

mod time;

fn main() {
    run_wgpu::main();
}
