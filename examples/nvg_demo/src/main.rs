mod blendish;
mod canvas;
mod run_gles;
mod run_wgpu;

mod time;

fn main() {
    if true {
        run_gles::main();
    } else {
        run_wgpu::main();
    }
}
