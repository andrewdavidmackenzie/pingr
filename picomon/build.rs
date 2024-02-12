//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[path = "src/pico_config.rs"]
mod pico_config;

use config;

const CONFIG_FILE_NAME: &str = "monitor.toml";

// Given a Config struct and a filename, generate that as a source file in OUT_DIR
fn generate_config(config: config::Config, filename: &str) {
    let out = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out);
    let out_file = out_dir.join(filename);
    let mut file = File::create(&out_file).unwrap();
    file.write_all(b"use crate::pico_config::Config;").unwrap();
    file.write_all(b"use crate::pico_config::MonitorSpec;")
        .unwrap();
    file.write_all(b"use crate::pico_config::ReportSpec;")
        .unwrap();
    file.write_all(b"pub(crate) const CONFIG: Config = ")
        .unwrap();

    file.write(b"Config {").unwrap();
    match &config.monitor {
        Some(monitor) => {
            file.write(b"    monitor: Some(").unwrap();
            match monitor {
                config::MonitorSpec::All => file.write(b"MonitorSpec::All").unwrap(),
                config::MonitorSpec::Connection => file.write(b"MonitorSpec::Connection").unwrap(),
                config::MonitorSpec::SSID(ssid, password) => file
                    .write(format!("MonitorSpec::SSID(\"{}\", \"{}\")", ssid, password).as_bytes())
                    .unwrap(),
            };
            file.write(b"    ),").unwrap()
        }
        None => file.write(b"    monitor: None,").unwrap(),
    };

    match &config.report {
        Some(report) => {
            file.write(b"    report: Some(").unwrap();

            file.write(b"ReportSpec {").unwrap();
            file.write(
                format!(
                    "        period_seconds: Some({}),",
                    report.period_seconds.unwrap()
                )
                .as_bytes(),
            )
            .unwrap();
            file.write(
                format!(
                    "        base_url: Some(\"{}\"),",
                    report.base_url.as_ref().unwrap()
                )
                .as_bytes(),
            )
            .unwrap();
            file.write(b"    }").unwrap();
            file.write(b"    ),").unwrap()
        }
        None => file.write(b"    report: None,").unwrap(),
    };
    file.write(b"}").unwrap();

    file.write_all(b";").unwrap();
}

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // TODO read monitor.toml and ssid.toml and generate a Config struct for picomon
    let config_file_path = config::find_config_file(CONFIG_FILE_NAME).unwrap();
    let mut config = config::read_config(&config_file_path).unwrap();
    config.monitor = Some(config::MonitorSpec::SSID(
        "MOVISTAR_8A9E".to_string(),
        "E68N8MA422GRQJQTPqjN".to_string(),
    ));
    generate_config(config, "config.rs");
}
