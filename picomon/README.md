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

## Useful Commands
Add a USB Modem (not USB HD mounted) pi pico as a USB HD device, so 
that I can then copy files to it without rebooting with the BOOTSEL pressed
- stty -f /dev/cu.usbmodem14301 1200

You should be able to copy a uf2 file to /Volumes/RPI-RP" using "cp" but I get an
error (and many others) from macos 14 on

However, this works
`ditto --norsrc --noextattr --noacl picow_blink.uf2 /Volumes/RPI-RP2`

people report that rsync may also work.

Copy executable to pico using SWD and debug probe
```commandline
sudo openocd -f interface/cmsis-dap.cfg -f target/rp2040.cfg -c "adapter speed 5000" -c "program ../target/thumbv6m-none-eabi/release/picomon verify reset exit"
```

Or with .cargo/config.toml having this line:
```
runner = "probe-rs run --chip RP2040 --protocol swd"
```

you can just use `cargo run`

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