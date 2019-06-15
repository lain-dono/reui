use gl_generator::{Registry, Api, Profile, Fallbacks, StaticGenerator};
use std::{env, fs::File, path::Path};

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(&Path::new(&dest).join("gl_bindings.rs")).unwrap();

    Registry::new(Api::Gles2, (3, 3), Profile::Core, Fallbacks::All, [])
        .write_bindings(StaticGenerator, &mut file)
        .unwrap();
}