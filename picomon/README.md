# Picomon

An implementation of monitor (like wimon) for the Raspberry Pi Pico W.

## Build and Run
Disconnect your RPi PicoW i fit is connected.
Press and hold the BOOTSEL button on the board while you connect via USB.
It should be mounted as a USB storage device (you may need to mount it on Linux).

Use `make run` which will build for the RPi ARM device and copy the built binary to the RPi Pico.
It will reboot and start running your binary.

See [config.toml](./.cargo/config.toml) for details. This is where the target is defined and
where the runner for cargo is set-up, to copy a UF2 file to the RPi Pico connected by USB.

Use `minicom` on the usbmodem device that should appear in /dev. You should see the log output on the terminal.

## Creating a UF2 file
Use the `elf2usb2` command which you can install using cargo.
`Usage: elf2uf2-rs <INPUT> [OUTPUT]`

Input should be the ELF file in `target/thumbv6m-none-eabi/release/picomon`
Let's make the output `picomon.uf2`

## Minicom
Add a USB Modem (not USB HD mounted) pi pico as a USB HD device, so
that I can then copy files to it without rebooting with the BOOTSEL pressed
- stty -f /dev/cu.usbmodem14301 1200

## Copying UF2 to Pi Pico
Boot the Pi Pico by disconnecting USB, pressing and holding the BOOTSEL button (the only one :-) ),
connecting the USB, then releasing BOOTSEL.

The Pi Pico will start in bootloader mode, and will read a UF2 file sent to it over USB.

On Mac, a new volume should appear in `/Volumes`. This is usually called `RPI-RP2`.
You might get a (helpful) alert that a new USB device was plugged in.
If not you can check using `ls /Volumes`.

Then you should be able to copy a uf2 file to /Volumes/RPI-RP" using "cp" but I (and many others on the
Internet) get an error  from macos 14.

However, this works
`ditto --norsrc --noextattr --noacl picomon.uf2 /Volumes/RPI-RP2`

people report that rsync may also work.

## Cargo run instead
If you have .cargo/config.toml with this line:
```
runner = "probe-rs run --chip RP2040 --protocol swd"
```

then you can just use `cargo run`

## GDB (not confirmed)
Starting gcb server:
`sudo openocd -f interface/cmsis-dap.cfg -f target/rp2040.cfg -c "adapter speed 5000"`

starting gdb client:
```commandline
gdb ../target/thumbv6m-none-eabi/debug/picomon
target remote localhost:3333
monitor reset init
continue
```
program should start running from boot.

starting lldb client
`lldb ../target/thumbv6m-none-eabi/debug/picomon`

then to connect to the gdb server:
```commandline
gdb-remote localhost:3333
```