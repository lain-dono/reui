#![feature(clamp)]

mod blendish;
mod canvas;
mod run_gles;
mod run_wgpu;

fn main() {
    if true {
        run_gles::main();
    } else {
        run_wgpu::main();
    }
}
