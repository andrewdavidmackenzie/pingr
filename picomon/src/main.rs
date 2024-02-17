//! For the RP Pico W on board LED.

#![no_std]
#![no_main]

extern crate alloc;

use core::fmt::Write;
use cyw43::Control;
use cyw43::NetDriver;
use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_net::dns::DnsSocket;
use embassy_net::{
    tcp::client::{TcpClient, TcpClientState},
    Stack, StackResources,
};
use embassy_rp::bind_interrupts;
use embassy_rp::flash::Async;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::USB;
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0};
use embassy_rp::pio::{InterruptHandler, Pio};
use embassy_rp::usb::{Driver, InterruptHandler as USBInterruptHandler};
use embassy_time::{Duration, Timer};
use embedded_alloc::Heap;
use faster_hex::hex_encode;
use heapless::String;
use log::{error, info};
use reqwless::{client::HttpClient, client::TlsConfig, client::TlsVerify, request::Method};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

mod pico_config;
use pico_config::{Config, MonitorSpec};

// The config generated by build.rs in OUT_DIR when compiling
mod config {
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}
use config::CONFIG;

#[global_allocator]
static HEAP: Heap = Heap::empty();

const FLASH_SIZE: usize = 2 * 1024 * 1024;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    USBCTRL_IRQ => USBInterruptHandler<USB>;
});

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

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<NetDriver<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
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

async fn wait_for_dhcp(stack: &Stack<NetDriver<'static>>) {
    info!("Waiting for DHCP...");
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");
}

async fn monitor_loop<'a>(
    device_id_hex: &[u8],
    ssid: &str,
    stack: &Stack<NetDriver<'static>>,
    mut control: Control<'_>,
    config: Config,
) {
    let mut report_count = 0;
    let period_seconds = config.report.period_seconds;
    let report_delay = Duration::from_secs(period_seconds);

    let base_url = config.report.base_url;
    let mut report_url: String<1024> = String::new();
    write!(
        &mut report_url,
        "{}/report/ongoing?device_id={}&connection=ssid%3D{}&period={}",
        base_url,
        core::str::from_utf8(&device_id_hex).unwrap(),
        ssid,
        period_seconds
    )
    .unwrap();
    info!("Reporting url = {report_url}");

    // To send to another async task:
    // Allocate with StaticCell. That one gives you a &'static mut T, without requiring
    // T to be Send/Sync.
    let client_state: TcpClientState<2, 1024, 1024> = TcpClientState::new();
    let client = TcpClient::new(&stack, &client_state);
    let dns = DnsSocket::new(&stack);
    //    let mut tls_rx = [0; 16384];
    //    let mut tls_tx = [0; 1024];
    let seed: u64 = 0x0123_4567_89ab_cdef;
    //    let mut client = HttpClient::new_with_tls(
    let mut client = HttpClient::new(
        &client,
        &dns,
        //        TlsConfig::new(seed, &mut tls_rx, &mut tls_tx, TlsVerify::None),
    );

    info!("Starting monitoring loop - will report every {period_seconds}s");

    loop {
        info!("Sending report #{}", report_count);
        control.gpio_set(0, true).await;

        let mut rx_buf = [0; 4096];
        let mut request = client.request(Method::GET, &report_url).await.unwrap();
        let response = request.send(&mut rx_buf).await;

        match response {
            Ok(response) => {
                info!("Response status: {:?}", response.status);
                if let Ok(payload) = response.body().read_to_end().await {
                    let s = core::str::from_utf8(payload).unwrap();
                    info!("Report sent: response = {}", s);
                }
            }
            Err(e) => {
                log::warn!("Error doing HTTP request: {:?}", e);
            }
        }

        control.gpio_set(0, false).await;

        report_count += 1;

        info!("Waiting");
        Timer::after(report_delay).await;
    }
}

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

    // Initialize the allocator BEFORE you use it
    {
        use core::mem::MaybeUninit;
        const HEAP_SIZE: usize = 4096;
        static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
        unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
    }

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;

    spawner.spawn(wifi_task(runner)).unwrap();
    spawner.spawn(logger_task(driver)).unwrap();

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    // Switch on led to show we are up and running
    control.gpio_set(0, true).await;

    // Get a unique device id - in this case an 8byte ID from flash rendered as hex string
    let mut device_id = [0; 8];
    let mut flash = embassy_rp::flash::Flash::<_, Async, { FLASH_SIZE }>::new(p.FLASH, p.DMA_CH1);
    flash.blocking_unique_id(&mut device_id).unwrap();
    let mut device_id_hex: [u8; 16] = [0; 16];
    hex_encode(&device_id, &mut device_id_hex).unwrap();
    log::info!(
        "Device ID = {}",
        core::str::from_utf8(&device_id_hex).unwrap()
    );

    let config = embassy_net::Config::dhcpv4(Default::default());

    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let resources = RESOURCES.init(StackResources::new());

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef;
    static STACK: StaticCell<Stack<NetDriver<'static>>> = StaticCell::new();
    let stack = STACK.init(Stack::new(net_device, config, resources, seed));

    spawner.spawn(net_task(stack)).unwrap();

    if let MonitorSpec::SSID(ssid, password) = CONFIG.monitor {
        match control.join_wpa2(ssid, password).await {
            Ok(_) => {
                info!("Joined wifi network: '{}'", ssid);
                wait_for_dhcp(stack).await;
                control.gpio_set(0, false).await;
                monitor_loop(&device_id_hex, ssid, stack, control, CONFIG).await;
            }
            Err(e) => error!("Error joining wifi: {:?}", e),
        }
    }
}
