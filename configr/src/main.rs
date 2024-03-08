use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;
use std::{env, fs, io};

const SSID_NAME_MARKER: &str = "$SSID_NAME::";
const SSID_PASS_MARKER: &str = "$SSID_PASS::";

fn main() -> io::Result<()> {
    if env::args().len() < 4 {
        print_help();
        exit(-1);
    }

    let mut args = env::args();
    let _ = args.next().unwrap();
    let ssid_name = args.next().unwrap();
    let ssid_pass = args.next().unwrap();
    let binary_filename = args.next().unwrap();

    match PathBuf::from(&binary_filename).canonicalize() {
        Ok(binary_path) => {
            edit_binary(&binary_path, &ssid_name, &ssid_pass)?;
            println!("Wrote modified binary '{}'", binary_filename);
            Ok(())
        }
        Err(e) => {
            eprintln!("Could not find the file: '{}'", binary_filename);
            Err(e)
        }
    }
}

fn replace_string(binary: &mut [u8], marker: &str, new_value: &str) -> io::Result<()> {
    let needle = marker.as_bytes();

    let mut iter = binary.windows(marker.len());
    let start = iter
        .position(|window| window == needle)
        .ok_or(io::Error::new(
            io::ErrorKind::NotFound,
            "Could not find first SSID_NAME Marker",
        ))?
        + marker.len();
    let end = start
        + iter
            .position(|window| window == needle)
            .ok_or(io::Error::new(
                io::ErrorKind::NotFound,
                "Could not find second SSID_NAME Marker",
            ))?
        - marker.len()
        + 1;

    let length = end - start;
    let new_value_padded = format!("{: <length$}", new_value);
    println!(
        "Replacing '{}' with '{new_value_padded}'",
        std::str::from_utf8(&binary[start..end]).unwrap(),
    );

    binary[start..end].copy_from_slice(new_value_padded.as_bytes());

    Ok(())
}

fn edit_binary(binary_path: &PathBuf, ssid_name: &str, ssid_pass: &str) -> io::Result<()> {
    println!("Attempting to edit the binary: '{}'", binary_path.display());

    let mut file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(binary_path)?;
    let metadata = fs::metadata(&binary_path).expect("unable to read metadata");
    let mut binary = vec![0; metadata.len() as usize];
    file.read(&mut binary).expect("buffer overflow");

    replace_string(&mut binary, SSID_NAME_MARKER, ssid_name)?;
    replace_string(&mut binary, SSID_PASS_MARKER, ssid_pass)?;

    file.write_all(&binary)
}

fn print_help() {
    println!(
        "Usage: {} {{ssid_name}} {{ssid_pass}} {{binary_filename}}",
        env!("CARGO_PKG_NAME")
    );
}
