use std::env;

fn print_pcsclite_error_message(target_os: &str) {
    if target_os == "linux" {
        eprintln!(
            "If you are using Debian (or a derivative), try:

    $ sudo apt install pkgconf libpcsclite-dev
"
        );
        eprintln!(
            "If you are using Arch (or a derivative), try:

    $ sudo pacman -S pkgconf pcsclite
"
        );
        eprintln!(
            "If you are using Fedora (or a derivative), try:

    $ sudo dnf install pkgconf pcsc-lite-devel
"
        );
    }
}

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
                println!(
                    "cargo:rustc-link-lib={}",
                    env::var("PCSC_LIB_NAME").unwrap_or_else(|_| "pcsclite".to_string())
                );
            } else {
                if let Err(err) = pkg_config::Config::new().atleast_version("1").probe("libpcsclite") {
                    eprintln!("Could not find a PCSC library.");
                    eprintln!(
                        "For the target OS `{}`, I tried to use pkg-config to find libpcsclite.",
                        target_os
                    );
                    eprintln!("The error given is: {}", err);
                    print_pcsclite_error_message(&target_os);
                    std::process::exit(1);
                }
            }
        }
    };
}
