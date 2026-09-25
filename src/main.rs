//! This example shows how to use RTC (Real Time Clock) in the RP2040 chip.

#![no_std]
#![no_main]

use core::{any::Any, borrow::Borrow, str};
use defmt::*;
use embassy_net::tcp::TcpSocket;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

embassy_rp::bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<embassy_rp::peripherals::PIO0>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<embassy_rp::peripherals::DMA_CH0>;

});

const WIFI_NETWORK: &str = "INEA-1997";
const WIFI_PASSWORD: &str = "zXXAXTkf";

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<
        'static,
        cyw43::SpiBus<
            embassy_rp::gpio::Output<'static>,
            cyw43_pio::PioSpi<'static, embassy_rp::peripherals::PIO0, 0>,
        >,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn st7735s() {}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    info!("Hello world!");

    let p = embassy_rp::init(Default::default());
    let mut rng = embassy_rp::clocks::RoscRng;

    // Configuring ST7735S controler (TFT-LCD).

    let clk = p.PIN_10;
    let mosi = p.PIN_11;
    let miso = p.PIN_12;
    let cs = p.PIN_16;

    let mut st7735s_spi_config = embassy_rp::spi::Config::default();
    st7735s_spi_config.frequency


    // Configuring cyw43.

    // Main firmare for WIFI chip. Copied to CYW43 RAM over SPI at boot.
    let fw = cyw43::aligned_bytes!("../cyw43-firmware/43439A0.bin");
    // Country Locale Manifest. Per-country TX power limits and allowed channels.
    let clm = cyw43::aligned_bytes!("../cyw43-firmware/43439A0_clm.bin");
    // NVRAM config variables. Small key/value strings (country code, ping/GPIO seturp for
    // the radio module, etc.) handled to the chip at initialization.
    let nvram = cyw43::aligned_bytes!("../cyw43-firmware/nvram_rp2040.bin");

    // GPIO 23 - OP wireless power on signal (board spec page 9)
    let pwr = embassy_rp::gpio::Output::new(p.PIN_23, embassy_rp::gpio::Level::Low);
    // GPIO 25 - OP wireless SPI CS - when high also enables GPIO29 ADC pin to read VSYS
    let cs = embassy_rp::gpio::Output::new(p.PIN_25, embassy_rp::gpio::Level::High);
    // This is programmable I/O
    let mut pio = embassy_rp::pio::Pio::new(p.PIO0, Irqs);
    // PIO for SPI interface to enable communication between chip and cyw43
    //
    // RP2040 PIO driver for the nonstandard half-duplex SPI used in the Pico W.
    // The PIO driver offloads SPI communication with the WiFi chip and improves throughput.
    let spi = cyw43_pio::PioSpi::new(
        &mut pio.common,
        pio.sm0,
        cyw43_pio::DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        p.PIN_24, // GPIO 24 - OP/IP wireless SPI data/IRQ
        p.PIN_29, // CLK
        embassy_rp::dma::Channel::new(p.DMA_CH0, Irqs), // DMA
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, cyw43_runner) = cyw43::new(state, pwr, spi, fw, nvram).await;
    spawner.spawn(unwrap!(cyw43_task(cyw43_runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // Configuring network stack.

    let config = embassy_net::Config::dhcpv4(Default::default());

    let seed = rng.next_u64();

    // Init network stack (max 3 sockets)
    static RESOURCES: StaticCell<embassy_net::StackResources<3>> = StaticCell::new();
    let (stack, net_runner) = embassy_net::new(
        net_device,
        config,
        RESOURCES.init(embassy_net::StackResources::new()),
        seed,
    );

    spawner.spawn(unwrap!(net_task(net_runner)));

    while let Err(err) = control
        .join(
            WIFI_NETWORK,
            cyw43::JoinOptions::new(WIFI_PASSWORD.as_bytes()),
        )
        .await
    {
        info!("join failed: {:?}", err);
    }

    info!("waiting for linl...");
    stack.wait_link_up().await;

    info!("waiting for DHCP...");
    stack.wait_config_up().await;

    info!("stack is up");

    let mut rx_buff = [0; 4096];
    let mut tx_buff = [0; 4096];
    let mut buff = [0; 4096];

    loop {
        let mut socket = embassy_net::tcp::TcpSocket::new(stack, &mut rx_buff, &mut tx_buff);
        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        control.gpio_set(0, false).await; // Pico W LED Off

        info!("listening on tcp:1234...");
        if let Err(err) = socket.accept(1234).await {
            warn!("accept error: {:?}", err);
            continue;
        }

        info!("received connection from {:?}", socket.remote_endpoint());
        control.gpio_set(0, true).await; // Picow W LED On

        loop {
            let n = match socket.read(&mut buff).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => n,
                Err(err) => {
                    warn!("read error: {:?}", err);
                    break;
                }
            };

            info!("rxd {}", core::str::from_utf8(&buff[..n]).unwrap());

            use embedded_io_async::Write;
            match socket.write_all(&buff[..n]).await {
                Ok(()) => {}
                Err(err) => {
                    warn!("write error: {:?}", err);
                    break;
                }
            }
        }
    }
}
