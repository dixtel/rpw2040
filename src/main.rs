//! This example shows how to use RTC (Real Time Clock) in the RP2040 chip.

#![no_std]
#![no_main]

use core::{any::Any, str};

use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIO0},
    pio::{InterruptHandler, Pio},
};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello world!");

    let p = embassy_rp::init(Default::default());

    // TODO; set rest of https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/wifi_scan.rs#L59

    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

    // GPIO23 - OP wireless power on signal (board spec page 9)
    let pwr = Output::new(p.PIN_23, Level::Low);
    // GPIO25 - OP wireless SPI CS - when high also enables GPIO29 ADC pin to read VSYS
    let cs = Output::new(p.PIN_25, Level::High);

    // This is programmable I/O, ref: https://hackspace.raspberrypi.com/articles/what-is-programmable-i-o-on-raspberry-pi-pico
    let mut pio = Pio::new(p.PIO0, Irqs);

    // So this PIO creates "SPI" interface to enable communication between chip and cyw43?
    //
    // From crate readme:
    // RP2040 PIO driver for the nonstandard half-duplex SPI used in the Pico W.
    // The PIO driver offloads SPI communication with the WiFi chip and improves throughput.
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24,  // GPIO24 - OP/IP wireless SPI data/IRQ
        p.PIN_29,  // CLK
        p.DMA_CH0, // DMA
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let router_bssid = &[0xee, 0xe8, 0x44, 0x85, 0x2f, 0x7c];

    let mut scanner = control.scan(Default::default()).await;
    while let Some(bss) = scanner.next().await {
        if let Ok(ssid_str) = str::from_utf8(&bss.ssid) {
            info!("scanned {} == {:x}", ssid_str, bss.bssid);
        }

        if bss.bssid == *router_bssid {
            info!("FOUND!!");
            break;
        } else {
            info!("NOT FOUND!!")
        }
    }
}
