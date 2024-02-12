//! For the RP Pico W on board LED.

#![no_std]
#![no_main]

use core::str;
use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::USB;
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::usb::{Driver, InterruptHandler as USBInterruptHandler};
use embassy_time::{Duration, Timer};
use log::info;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

mod config;
use config::MonitorSpec;
use config::DEFAULT_CONFIG;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    USBCTRL_IRQ => USBInterruptHandler<USB>;
});

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    >,
) -> ! {
    runner.run().await
}

/*
async fn list_ssid(control: &mut Control<'_>) {
    let mut scanner = control.scan().await;
    let bss = scanner.next().await.unwrap();
    match str::from_utf8(&bss.ssid) {
        Ok(ssid_str) => info!("{}", ssid_str), /* info!("scanned {} == {:x}", ssid_str, bss.bssid),*/
        Err(_) => log::info!("Could not scan wifi"),
    }
}
*/

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);
    let fw = include_bytes!("../assets/43439A0.bin");
    let clm = include_bytes!("../assets/43439A0_clm.bin");
    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (_net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

    spawner.spawn(wifi_task(runner)).unwrap();
    spawner.spawn(logger_task(driver)).unwrap();

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // Switch on led to show we are up and running
    control.gpio_set(0, true).await;

    if let Some(MonitorSpec::SSID(ssid, password)) = DEFAULT_CONFIG.monitor {
        match control.join_wpa2(ssid, password).await {
            Ok(_) => {
                log::info!("Joined wifi");
                let mut counter = 0;
                let delay = Duration::from_secs(1);
                loop {
                    info!("LED on {}", counter);
                    control.gpio_set(0, true).await;
                    Timer::after(delay).await;

                    info!("LED off {}", counter);
                    control.gpio_set(0, false).await;
                    Timer::after(delay).await;

                    counter += 1;
                }
            }
            Err(_) => info!("Error joining wifi"),
        }
    }
}
