# Pico W 
- RP2040
- board specification: https://datasheets.raspberrypi.com/picow/pico-w-datasheet.pdf
- processor specification: https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf
- https://datasheets.raspberrypi.com/pico/getting-started-with-pico.pdf
- rust architecture for armv6-M - thumbv6m-none-eabi
- embassy documentation: https://embassy.dev/book/
- embassy exmaples: https://github.com/embassy-rs/embassy/tree/main/examples/rp/src/bin
- PIO stands for "Programmable Inpint/Output"
- There are two PIO blocks in Pico W
- PIO0 is used in setup for CYW43

# Power
- Powering through VSYS [9]

# SPI
Two hardware SPI: SPI0 and SPI1 (SCL, MISO, MOSI, CS).
SPI0: Pins GPIO_2 -> GPIO_5 
SPI1: Pins GPIO_10 -> GPIO_13
Alternative default pin locations can be configured through software.

## Pins
| Pin | Name     | Note                                              |
| 23  | 23       | Power for cyw43                                   |
| 27  | 25       | cyw43 control (on-board led in pico without wifi) |
| 40  | VBUS     | Micro-USB input voltage.  5V +- 10%               |
| 39  | VSYS     | Main system input volated. Used to generate 3.3V   
                   for RP2040 and GPIO Min 1.8V, max 5.5V            |
| 38  | GND      |                                                   |
| 37  | 3V3_EN   | Pull low to power off the Pico                    |
| 36  | 3VE(OUT) | Can be used to power external circuits. Max 300mA |
    
## problems
Running `probe-rs run --chip rp2040 target/thumbv6m-none-eabi/debug/rpw2040`
 WARN probe_rs::flashing::loader: No loadable segments were found in the ELF file.
Error: No loadable ELF sections were found.

## probe-rs info --protocol swd --chip rp2040

Probing target via SWD

ARM Chip:
Debug Port: Version 2, MINDP, Designer: Raspberry Pi Trading Ltd, Part: 0x1002, Revision: 0x0

Debugging RISC-V targets over SWD is not supported. For these targets, JTAG is the only supported protocol. RISC-V specific information cannot be printed.
Debugging Xtensa targets over SWD is not supported. For these targets, JTAG is the only supported protocol. Xtensa specific information cannot be printed.

## How to run
- cargo run

## CYW43
Pins:
- GPIO_23: Power off/on control signal.
- GPIO_24: Shared SPI data and IWQ betwwen RP2040 chip and CYW43.
- GPIO_25: SPI CS. Chip power state monitoring.
- VSYS: Can be read safely when and active external SPI bus transaction is not underway.

Avoid repurposing GPIO 12, 24 and 25 for general-purpose external SPI transactions.


Due to pin limitations, some of the wireless interface pins are shared. The CLK is shared with
VSYS monitor, so only when there isn’t an SPI transaction in progress can VSYS be read via the ADC.
The Infineon CYW43439 DIN/DOUT and IRQ all share one pin on the RP2040. Only when an
SPI transaction isn’t in progress is it suitable to check for IRQs. The interface typically runs
at 33 MHz.

## Playground
- check when the clock time is stored and read it to messure performance of given function

# SPI - Serial Peripheral Interface
- Synchronous, full fuplex interface
- 3 or 4 wire

Main                   Subnode      Note 
CSX  (or CS)      ->   CS           (Chip Select)
SCL  (or SCK)     ->   SCLK         (Serial Clock)
MOSI (or SDA)     ->   SDI          (Master output, slave input)
MISO (or SDO)     <-   SDO          (Master input, slave input)
D/CX (or DC / RS) ->                (Data/Command selection, 4-wire SPI)

Glosary:
- Main: Generates the clock signal.
- MOSI: Main out, subnode in. Transmits data from main to subnode.
- MISO: Main in, subnode out. Transmits data from subnode to main.
- SCLK or CLK: Clock. Data is transmitted depending on CPOL.
- CS, or SS:  Chip Select or Slave Select. Usually is active low and is pulled high to
  disconnect subnode from SPI.
  When multiple subnodes are used, an individual chip select signal is rwquired from the main. 
- CPOL: Clock Polarity. The bit sets the polarity of the clock during idle state. The idle state
  is defined as the period when CS is high and transistioning to low at the start of
  transmission or when CS is low and transistioning to high at the end of transmission.
  * Value 0 means that the idle state is on clock active low.
  * Value 1 means that the idle state is on clock active high. 
- CPHA: Clock Phase. Depending on the bit the rising or falling clock edge is used to
  sample and/or shift the data.
  Indicates on which rising/falling edge data is sampled/shifted.
  * Value 0 means that data is sampled and then shifted.
  * Value 1 means that data is shifted and then sampled.

Setup:
1. Send clock signal.
2. Chip select (usually chip select is active low).
3. Select the CPOL and CPHA, as per requirement of the subnode.


SPI Modes:
- Mode 0 (CPOL 0, CPHA 0) Logic low;  Sampled on rising;  Shifted on falling
- Mode 1 (CPOL 0, CPHA 1) Logic low;  Sampled on falling; Shifted on rising
- Mode 2 (CPOL 1, CPHA 0) Logic high; Sampled on falling; Shifted on rising
- Mode 3 (CPOL 1, CPHA 1) Logic high; Sampled on rising;  Shifted on falling

Multi-subnode configuration types:
- Regular: The main is connected independetly to each subnode.
- Daisy-chain: The main is connected only to first subnode. The previous subnode is connected to
  the next subnode and the last subnode is connected to main. Not every subnode can suuport
  daisy-chain configuration.

# ST7735S
TFT-LCD Display. 128x162 18bpp resultuion, 1.8'

```
+--------------------------------------------------------------------------------------------------+
|                                       3-Pin Serial Mode                                          |
|                                                                                                  |
|          +----------------------------+              +----------------------------+              |
|          |          PICO W            |              |          ST7735S           |              |
|          |                            |              |                            |              |
|          |               3.3V 3V3_OUT | -----------> | VDD                        |              |
|          |                    GND     | ------------ | GND                        |              |
|          |                            |              |                            |              |
|          |               SCL  GPIO_10 | -----------> | SCL    set clock           |              |
|          |               MISO GPIO_11 | -----------> | SDA    slave data          |              |
|          |               CS   GPIO_13 | -----------> | CS     clock set           |              |
|          |                            |              |                            |              |
|          |               PWM   GPIO_3 | -----------> | BLK    backlight (PWM)     |              |
|          |                     GPIO_4 | -----------> | RST    reset               |              |
|          |                     GPIO_5 | -----------> | DC     data control        |              |
|          |                            |              |                            |              |
|          +----------------------------+              +----------------------------+              |
+--------------------------------------------------------------------------------------------------+
```

Pins:
- BLK: Backlight control (optional PWM dimming)
- CS: Chip Select
- DC: Data/Command control
- RST: Reset
- SDA: Serial Data Line
- SCL: Serial Clock Line
- VDD: Power supply 3.3V-5V
- GND

Interface Type Selection: P68, IM3, IM1, IM0.

Hardware interface select:
- IM2=0 4WSPI=0: 3-line SPI, 9 bits
- IM2=0 4WSPI=1: 4-line SPI, 8 bits

SPI configuration:
- CS is 0 on data transmission (CS is active low, !CS, CS#)
- CPOL is 1

Behaviour:
- D/CX? + 8-bit Big-endian (BE)
- When CSX is high (SDA and SCK has no effect).
- Falling edge on CSX enables SPI and indicates start of transmission.
- D/CX is low then the TB (Transmission byte) is a command.
- D/CX is high then the TB is a data.
- SDA is sampled at the rising edge of SCL

Read:
- Flow:
  1. Send command 'read ID or 'register'.
  2. Read data.
  3. Set CSX to 1.
- The driver samples data on rising edge of SCL and shifts data at the falling edge thus
  the micro controler is supported to read data at the rising edge of SCL.

Programming:
- Data formats:
  * 4k -> 4-4-4 bit (2*^12=4096 color mapping table, color depth information)
  * 65k -> 5-6-5 bit
  * 262k -> 6-6-6 bit
- 4k:
  * RESX is 1
  * IM is 0
  * SDA is 1
  * Data: (1 + R + G + 1 + B) + (R + 1 + G + B) + ...
- Before writing to RAM, a windows must be defined that will be written.
  * Command registers:
    - XS, YS - start address
    - XE, YE - end address
  * Writing for the whole screen:
    - XS = 0, YS = 0
    - XE = 127 (83h), YE = 161 (A1h)
- Addressing:
  - Horizontal (V=0)
    - X-address increments after each byte, wraps at XE to XS and Y increments to the next row.
  - Vertical (MV=1)
    - Y-address increments after each byte, wraps at YE to YE and X increments to the next column.
  - When X=XE and Y=YE then (X, Y) is set to (XS, YS)
- Vertical scrolling:
  * Vertical Scrolling Definition (33h)
  * Vertical Scrolling Start Address (37h)
- Paremetrs
  - MADCTL: MV, MX, MY
    * MV=MX=MY=0: Normal display data direction
  - CASET, RASET

Power ON/OFF Sequence ([5]):
- VDDI and VDD can be applied in any order.
- VDD and VDDI can be powered down in any order.
- During power off:
  * if LCD is in the Sleep Out Mode:
    - VDD and VDDI must be powered down minimium 120msec after RESX hax been released.
  * if LCD is in the Sleep In mode:
    - VDDI or VDD can be powered down minimium 0msec after RESC hac been relaesed.
- CSC can be applied at eny timing or dcan be permanently grounded.
- RESX has priority over CSX.

Power levels ([6]):
- Normal Mode ON (Full Display) + Idle Mode Off + Sleept Out
  * 262,144 colors.
- Partial Mode ON (Part Display), Idle Mode off, Sleep Out
  * 262,144 colors.
- Normal Mode ON (Full Display), Idle Mode ON, Sleep Out
  * 8 colors.
- Partial Mode ON (Part Display), Idle Mode ON, Sleep Out
  * 8 colors.
- Sleep In
  * Only MCU and mwmory works with VDDI power supply.
- Power Off
  * VDD and VDDI are removed

Reset Table [7].
List of commands [8].

# Footnotes
- [1] Serial Interface Characteristics ./ST7735S.PDF#35
- [2] Interface Write Protocol ./ST7735S.PDF#45
- [3] Write Data for 12-bit Pixel ./ST7735S.PDF#62
- [4] Frame Date Write Direction ./ST7735S.PDF#77
- [5] Power on/off sequence ./ST7735S.PDF#85
- [6] Power Flow Chart ./ST7735S.PDF#88
- [7] Reset table ./ST7735S.PDF#89
- [8] List of commands ./ST7735S.PDF#104
- [9] https://penguintutor.com/electronics/pico-power#:~:text=Powering%20the%20Raspberry%20Pi%20Pico%20through%20VSYS

