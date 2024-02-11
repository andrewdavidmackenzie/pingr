
Add a USB Modem (not USB HD mounted) pi pico as a USB HD device, so 
that I can then copy files to it without rebooting with the bootsel pressed
- stty -f /dev/cu.usbmodem14301 1200

You should be able to copy a uf2 file to /Volumes/RPI-RP" using "cp" but I get an
error (and many others) from macos 14 on

However, this works
`ditto --norsrc --noextattr --noacl picow_blink.uf2 /Volumes/RPI-RP2`

people report that rsync may also work.