#![warn(clippy::all)]

mod blendish;
mod canvas;
mod run_gles;
mod run_wgpu;

mod time;

fn main() {
    if false {
        run_gles::main();
    } else {
        run_wgpu::main();
    }
}
