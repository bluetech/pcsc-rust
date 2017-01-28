extern crate pkg_config;

use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    let target = target.split('-').collect::<Vec<_>>();

    // Prefer the built-in service/library if available, otherwise try
    // libpcsclite.
    match (target.get(1), target.get(2)) {
        (Some(&"pc"), Some(&"windows")) => {
            // Note we also have to specify this above the extern {} for
            // some reason (see comment there).
            println!("cargo:rustc-link-lib=dylib=winscard");
        }

        (Some(&"apple"), Some(&"darwin")) => {
            println!("cargo:rustc-link-lib=framework=PCSC");
        }

        _ => {
            pkg_config::Config::new().atleast_version("1").probe("libpcsclite").unwrap();
        }
    };
}
