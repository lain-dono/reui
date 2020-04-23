#![warn(clippy::all)]

mod blendish;
mod canvas;
mod run_gles;
mod run_wgpu;

mod time;

fn main() {
    //run_gles::main();
    run_wgpu::main();
}
