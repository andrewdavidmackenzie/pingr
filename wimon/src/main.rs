use std::{env, io};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

use service_manager::{
    ServiceInstallCtx, ServiceLabel, ServiceManager, ServiceStartCtx, ServiceStopCtx,
    ServiceUninstallCtx,
};

use config::MonitorSpec;

mod monitor;

const CONFIG_FILE_NAME: &str = "monitor.toml";

const SERVICE_NAME: &str = "net.mackenzie-serres.pingr.wimon";

fn main() -> Result<(), io::Error> {
    let service_name: ServiceLabel = SERVICE_NAME.parse().unwrap();

    let args: Vec<_> = env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        None => {
            let config_file_path = config::find_config_file(CONFIG_FILE_NAME)?;
            run(&config_file_path)?;
        }
        Some("install") => install_service(&service_name, &args[0])?,
        Some("uninstall") => uninstall_service(&service_name)?,
        _ => eprintln!("Invalid argument(s): '{}'", &args[1..].join(", ")),
    }

    Ok(())
}

fn run(config_file_path: &PathBuf) -> Result<(), io::Error> {
    let config = config::read_config(config_file_path)?;
    println!(
        "Config file loaded from: \"{}\"",
        config_file_path.display()
    );
    println!(
        "Monitor: {:?}",
        config.monitor.as_ref().unwrap_or(&MonitorSpec::Connection)
    );

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

fn get_service_manager() -> Result<Box<dyn ServiceManager>, io::Error> {
    // Get generic service by detecting what is available on the platform
    let manager = <dyn ServiceManager>::native()
        .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Could not create ServiceManager"))?;

    Ok(manager)
}

// This will install the binary as a user level service and then start it
fn install_service(service_name: &ServiceLabel, path_to_exec: &str) -> Result<(), io::Error> {
    let manager = get_service_manager()?;
    let exec_path = PathBuf::from(path_to_exec).canonicalize()?;
    // Run from dir where exec is for now, so it should find the config file in ancestors path
    let exec_dir = exec_path
        .parent()
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not get exec dir",
        ))?
        .to_path_buf();

    // Install our service using the underlying service management platform
    manager.install(ServiceInstallCtx {
        label: service_name.clone(),
        program: exec_path,
        args: vec![],
        contents: None, // Optional String for system-specific service content.
        username: None, // Optional String for alternative user to run service.
        working_directory: Some(exec_dir),
        environment: None, // Optional list of environment variables to supply the service process.
        autostart: true, // autostart on reboot
    })?;

    // Start our service using the underlying service management platform
    manager.start(ServiceStartCtx {
        label: service_name.clone(),
    })?;

    println!(
        "'service '{}' ('{}') installed and started",
        service_name, path_to_exec
    );

    Ok(())
}

// this will stop any running instance of the service, then uninstall it
fn uninstall_service(service_name: &ServiceLabel) -> Result<(), io::Error> {
    let manager = get_service_manager()?;

    // Stop our service using the underlying service management platform
    manager.stop(ServiceStopCtx {
        label: service_name.clone(),
    })?;

    println!(
        "service '{}' stopped. Waiting for 10s before uninstalling",
        service_name
    );
    std::thread::sleep(Duration::from_secs(10));

    // Uninstall our service using the underlying service management platform
    manager.uninstall(ServiceUninstallCtx {
        label: service_name.clone(),
    })?;

    println!("service '{}' uninstalled", service_name);

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use config::{Config, MonitorSpec};

    use super::CONFIG_FILE_NAME;

    #[test]
    fn bundled_spec() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let root_dir = manifest_dir
            .parent()
            .ok_or("Could not get parent dir")
            .expect("Could not get parent dir");
        let config_string = std::fs::read_to_string(root_dir.join(CONFIG_FILE_NAME)).unwrap();
        let config: Config = toml::from_str(&config_string).unwrap();
        assert_eq!(config.monitor, Some(MonitorSpec::Connection));
        assert_eq!(config.report.unwrap().period_seconds, Some(60));
    }
}
