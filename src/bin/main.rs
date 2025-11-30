#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{with_timeout, Duration, Timer};
use esp_bootloader_esp_idf::esp_app_desc;
use esp_csi_rs::{collector::CSISniffer, config::CSIConfig};
use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use esp_println as _;
use esp_println::println;
use esp_wifi::{init, EspWifiController};

esp_app_desc!();

extern crate alloc;

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // Configure System Clock
    let config = esp_hal::Config::default().with_cpu_clock(esp_hal::clock::CpuClock::max());
    // Take Peripherals
    let peripherals = esp_hal::init(config);

    // Allocate some heap space
    esp_alloc::heap_allocator!(size:72 * 1024);

    // Initialize Embassy
    let timg1 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timg1.timer0);

    // Instantiate peripherals necessary to set up  WiFi
    let timer1 = esp_hal::timer::timg::TimerGroup::new(peripherals.TIMG0);
    let wifi = peripherals.WIFI;
    let timer = timer1.timer0;
    let rng = Rng::new(peripherals.RNG);

    // Initialize WiFi Controller
    let init = &*mk_static!(EspWifiController<'static>, init(timer, rng).unwrap());

    // Instantiate WiFi controller and interfaces
    let (controller, interfaces) = esp_wifi::wifi::new(&init, wifi).unwrap();

    println!("WiFi Controller Initialized");

    // Create a Sniffer CSI Collector
    // Don't filter any MAC addresses out
    let mut csi_coll_snif = CSISniffer::new(CSIConfig::default(), controller).await;

    // Initialize CSI Collector
    csi_coll_snif.init(interfaces, &spawner).await.unwrap();

    // Start Collection
    csi_coll_snif.start_collection().await;

    // Collect for 2 Seconds
    with_timeout(Duration::from_secs(20), async {
        loop {
            let csi_data = csi_coll_snif.get_csi_data().await;
            println!("CSI Data: {:x?}", csi_data);
        }
    })
    .await
    .unwrap_err();

    // Stop Collection
    csi_coll_snif.stop_collection().await;

    println!("Collection Ended");

    loop {
        Timer::after(Duration::from_secs(1)).await
    }
}