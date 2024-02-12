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
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

#[path = "src/config.rs"]
mod config;
use config::Config;
use config::MonitorSpec;
use config::ReportSpec;

const DEFAULT_CONFIG: Config = Config {
    monitor: Some(MonitorSpec::SSID("MOVISTAR_8A9E", "E68N8MA422GRQJQTPqjN")),
    report: Some(ReportSpec {
        period_seconds: Some(60),
        base_url: Some("https://collectr.mackenzie-serres.workers.dev"),
    }),
};

impl Display for MonitorSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorSpec::All => write!(f, "MonitorSpec::All")?,
            MonitorSpec::Connection => write!(f, "MonitorSpec::Connection")?,
            MonitorSpec::SSID(ssid, password) => {
                write!(f, "MonitorSpec::SSID(\"{}\", \"{}\")", ssid, password)?
            }
        }
        Ok(())
    }
}

impl Display for ReportSpec {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "ReportSpec {{")?;
        writeln!(
            f,
            "        period_seconds: Some({}),",
            self.period_seconds.unwrap()
        )?;
        writeln!(f, "        base_url: Some(\"{}\"),", self.base_url.unwrap())?;
        write!(f, "    }}")
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Config {{")?;
        match &self.monitor {
            Some(m) => writeln!(f, "    monitor: Some({}),", m)?,
            None => writeln!(f, "    monitor: None,")?,
        };
        match &self.report {
            Some(r) => writeln!(f, "    report: Some({}),", r)?,
            None => writeln!(f, "    report: None,")?,
        }
        write!(f, "}}")
    }
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

    // read monitor.toml and ssid.toml and generate a Config struct for picomon */
    let out = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out);
    let out_file = out_dir.join("default_config.rs");
    println!("cargo:warning=Out File is {}", out_file.display());
    let mut file = File::create(&out_file).unwrap();
    file.write_all(b"use crate::config::Config;").unwrap();
    file.write_all(b"use crate::config::MonitorSpec;").unwrap();
    file.write_all(b"use crate::config::ReportSpec;").unwrap();
    file.write_all(format!("pub(crate) const CONFIG: Config = {};", DEFAULT_CONFIG).as_bytes())
        .expect("Could not write Config to source file in OUT_DIR");
}
