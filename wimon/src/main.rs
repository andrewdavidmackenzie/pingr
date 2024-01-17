use std::io;
use std::sync::mpsc::channel;

use ctrlc;

use config::MonitorSpec;

mod config;
mod monitor;

fn main() -> Result<(), io::Error> {
    let config_file_path = config::find_config_file(config::CONFIG_FILE_NAME)?;
    let config = config::read_config(&config_file_path)?;
    println!("Config file loaded from: \"{}\"", config_file_path.display());
    println!("Monitor: {:?}", config.monitor_spec.as_ref().unwrap_or(&MonitorSpec::Connection));

    let (tx, rx) = channel();
    ctrlc::set_handler(move || {
        println!("Control-C captured, sending Stop report");
        tx.send(()).expect("Could not send signal on channel.")
    })
        .expect("Error setting Ctrl-C handler");

    monitor::monitor_loop(config, rx)?;

    println!("Exiting");

    Ok(())
}
