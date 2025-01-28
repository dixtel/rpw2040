# Pico W RP204
- board specification: https://datasheets.raspberrypi.com/picow/pico-w-datasheet.pdf
- processor specification: https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf
- https://datasheets.raspberrypi.com/pico/getting-started-with-pico.pdf

- rust architecture for armv6-M - thumbv6m-none-eabi
- embassy documentation: https://embassy.dev/book/
- embassy exmaples: https://github.com/embassy-rs/embassy/tree/main/examples/rp/src/bin
    
# problems
Running `probe-rs run --chip rp2040 target/thumbv6m-none-eabi/debug/rpw2040`
 WARN probe_rs::flashing::loader: No loadable segments were found in the ELF file.
Error: No loadable ELF sections were found.

# probe-rs info --protocol swd --chip rp2040

Probing target via SWD

ARM Chip:
Debug Port: Version 2, MINDP, Designer: Raspberry Pi Trading Ltd, Part: 0x1002, Revision: 0x0

Debugging RISC-V targets over SWD is not supported. For these targets, JTAG is the only supported protocol. RISC-V specific information cannot be printed.
Debugging Xtensa targets over SWD is not supported. For these targets, JTAG is the only supported protocol. Xtensa specific information cannot be printed.

# How to run
- cargo run


# CYW43
- PIO stands for "Programmable Inpint/Output"
 - There are two PIO blocks in Pico W
 - PIO0 is used in setup for CYW43
