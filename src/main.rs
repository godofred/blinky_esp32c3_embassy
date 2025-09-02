use embassy_executor::Spawner;
use embassy_time::{Timer};
use esp_idf_svc::hal::gpio::{
    self, OutputPin
};
use esp_idf_svc::hal::gpio::{PinDriver};
use esp_idf_svc::hal::peripherals;

use esp_idf_svc::hal::ledc::{Resolution};
use esp_idf_svc::hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_svc::hal::units::Hertz;

mod wifi_connection;

fn init() {
    let link_patches = esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    init();

    let peripherals = peripherals::Peripherals::take().unwrap();

    log::info!("Hello, world!"); 

    let led_gpio = peripherals.pins.gpio8.downgrade_output();
    let led_pin = gpio::PinDriver::output(led_gpio).unwrap();
    spawner.spawn(blink_task(led_pin)).unwrap();

    let _wifi = wifi_connection::async_connect_wifi(peripherals.modem).await;

    std::mem::forget(_wifi);
}

#[embassy_executor::task(pool_size = 2)]
async fn blink_task(mut pin: PinDriver<'static, gpio::AnyOutputPin, gpio::Output>) {
    log::info!("Starting blink task on GPIO {}!", pin.pin());
    loop {
        pin.set_high().unwrap();
        Timer::after_millis(500).await;
        pin.set_low().unwrap();
        Timer::after_millis(500).await;
    }
}
