extern crate pkg_config;

use std::env;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS")
        .expect(r#"The CARGO_CFG_TARGET_OS environment is not set in the build script."#);

    // Prefer the built-in service/library if available, otherwise try
    // libpcsclite.
    match &*target_os {
        "windows" => {
            // Note we also have to specify this above the extern {} for
            // some reason (see comment there).
            println!("cargo:rustc-link-lib=dylib=winscard");
        }

        "macos" => {
            println!("cargo:rustc-link-lib=framework=PCSC");
        }

        _ => {
            if let Ok(lib_dir) = env::var("PCSC_LIB_DIR") {
                println!("cargo:rustc-link-search=native={}", lib_dir);
                println!("cargo:rustc-link-lib={}", env::var("PCSC_LIB_NAME").unwrap_or("pcsclite".to_string()));
            } else {
                pkg_config::Config::new()
                    .atleast_version("1")
                    .probe("libpcsclite")
                    .expect(&format!(
                        r#"Could not find a PCSC library.
For the target OS `{}`, I tried to use pkg-config to find libpcsclite.
Do you have pkg-config and libpcsclite configured for this target?"#,
                        target_os
                    ));
            }
        }
    };
}
