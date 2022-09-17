use std::{env, path::PathBuf};

fn main() {
    let bindings = bindgen::Builder::default()
        .header("src/ip.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .unwrap();

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("ip_bindings.rs"))
        .unwrap();
}
