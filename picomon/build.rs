//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use config::SsidSpec;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[path = "src/pico_config.rs"]
mod pico_config;

const CONFIG_FILE_NAME: &str = "monitor.toml";
const SSID_FILE_NAME: &str = "ssid.toml";
const SSID_NAME_LENGTH: usize = 32;
const SSID_PASS_LENGTH: usize = 63;

// Given a Config struct and a filename, generate that as a source file in OUT_DIR
fn generate_config(config: config::Config, filename: &str, ssid: SsidSpec) {
    let out = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out);
    let out_file = out_dir.join(filename);
    let mut file = File::create(out_file).unwrap();
    file.write_all(b"use crate::pico_config::Config;\n")
        .unwrap();
    file.write_all(b"use crate::pico_config::MonitorSpec;\n")
        .unwrap();
    file.write_all(b"use crate::pico_config::ReportSpec;\n")
        .unwrap();
    file.write_all(b"pub(crate) const MARKER_LENGTH : usize = \"$SSID_NAME::\".len();\n")
        .unwrap();
    file.write_all(b"pub(crate) const SSID_NAME_LENGTH : usize = 32;\n")
        .unwrap();
    file.write_all(b"pub(crate) const SSID_PASS_LENGTH : usize = 63;\n")
        .unwrap();
    // SSID Names can be upto 32 ASCII characters plus 24 for markers = 56
    // right pad the provided string with spaces upto 32 ASCII characters (bytes)
    file.write_all(
        format!(
            "pub(crate) const SSID_NAME : &str = \"$SSID_NAME::{: <SSID_NAME_LENGTH$}$SSID_NAME::\";\n",
            ssid.ssid_name
        )
        .as_bytes(),
    )
    .unwrap();
    // SSID Passwords can be upto 63 ASCII characters plus 24 for markers = 87
    file.write_all(
        format!(
            "pub(crate) const SSID_PASS : &str = \"$SSID_PASS::{: <SSID_PASS_LENGTH$}$SSID_PASS::\";\n",
            ssid.ssid_pass
        )
        .as_bytes(),
    )
    .unwrap();

    file.write_all(b"pub(crate) const CONFIG: Config = ")
        .unwrap();

    file.write_all(b"Config {").unwrap();
    match &config.monitor {
        Some(monitor) => {
            file.write_all(b"    monitor: ").unwrap();
            match monitor {
                config::MonitorSpec::All => file.write(b"MonitorSpec::All").unwrap(),
                config::MonitorSpec::Connection => file.write(b"MonitorSpec::Connection").unwrap(),
            };
            file.write_all(b"    ,").unwrap()
        }
        None => {
            // TODO fail or substitute a default value here
            file.write_all(b"    monitor: None,").unwrap()
        }
    };

    match &config.report {
        Some(report) => {
            file.write_all(b"    report: ").unwrap();
            file.write_all(b"ReportSpec {").unwrap();
            file.write_all(
                format!(
                    "        period_seconds: {},",
                    report.period_seconds.unwrap()
                )
                .as_bytes(),
            )
            .unwrap();
            file.write_all(
                format!(
                    "        base_url: \"{}\",",
                    report.base_url.as_ref().unwrap()
                )
                .as_bytes(),
            )
            .unwrap();
            file.write_all(b"    }").unwrap();
            file.write_all(b"    ,").unwrap()
        }
        None => {
            // TODO fail or substitute a default value here
            file.write_all(b"    report: None,").unwrap()
        }
    };
    file.write_all(b"}").unwrap();

    file.write_all(b";").unwrap();
}

fn main() {
    // Put `memory.x` in our output directory and ensure it's on the linker search path.
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

    // Generate a pico_config::Config struct for picomon from the monitor.toml file
    let config_file_path =
        config::find_config_file(CONFIG_FILE_NAME).expect("Could not find monitor.toml file");
    let config = config::read_config(&config_file_path).unwrap();
    // rebuild if ../monitor.toml changes
    println!("cargo:rerun-if-changed=../monitor.toml");

    let ssid_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join(SSID_FILE_NAME);
    let ssid_spec = config::read_ssid(&ssid_path).expect("Could not find ssid.toml file");

    generate_config(config, "config.rs", ssid_spec);
}
