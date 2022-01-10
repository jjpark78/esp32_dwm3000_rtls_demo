extern crate bindgen;
extern crate cc;

use std::env;
use std::path::Path;
use std::path::PathBuf;

use embuild::{
    // self, bindgen, bingen,
    self,
    build::{CfgArgs, LinkArgs},
    // cargo, symgen,
};

fn main() -> anyhow::Result<()> {
    // Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
    LinkArgs::output_propagated("ESP_IDF")?;

    let cfg = CfgArgs::try_from_env("ESP_IDF")?;

    cfg.output();

    //for DWM3000 Module
    cc::Build::new()
        .file("./decawave_api/port.c")
        .file("./decawave_api/deca_spi.c")
        .file("./decawave_api/deca_device.c")
        .file("./decawave_api/deca_mutex.c")
        .file("./decawave_api/deca_sleep.c")
        .file("./decawave_api/shared_functions.c")
        .file("./decawave_api/MAC_802_15_8/mac_802_15_8.c")
        .file("./decawave_api/MAC_802_15_4/mac_802_15_4.c")
        .includes(Some(Path::new("./decawave_api")))
        .define("USE_ZLIB", None)
        .compile("libdwm3000.a");

    let deca_bindings = bindgen::Builder::default()
        .clang_arg("-I./decawave_api")
        .header("./decawave_api/wrapper.h")
        .generate()
        .expect("Unable to generate decawave's bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    deca_bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write bindings!");

    println!("cargo:rustc-link-lib=dwm3000");
    println!("cargo:rerun-if-changed=./decawave_api/wrapper.h");

    Ok(())
}
