use std::error::Error;

use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::modem::Modem;
use esp_idf_svc::hal::spi::SPI3;
use esp_idf_svc::hal::spi::{config::Config, SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::sntp::{self, SyncStatus};
use esp_idf_svc::wifi::{BlockingWifi, ClientConfiguration, Configuration, EspWifi};

use crate::{MySpiDriver, CONFIG};

pub use crate::display::init::*;

pub fn esp() {
    // It is necessary to call this function once. Otherwise some patches
    // to the runtime implemented by esp-idf-sys might not link properly.
    // See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Esp initialized!");
}

pub fn wifi(modem: Modem) -> Result<BlockingWifi<EspWifi<'static>>, Box<dyn Error>> {
    let sysloop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let mut wifi = BlockingWifi::wrap(EspWifi::new(modem, sysloop.clone(), Some(nvs))?, sysloop)?;

    let client = Configuration::Client(ClientConfiguration {
        ssid: heapless::String::try_from(CONFIG.wifi_ssid).unwrap(),
        password: heapless::String::try_from(CONFIG.wifi_psk).unwrap(),
        auth_method: esp_idf_svc::wifi::AuthMethod::None,
        ..Default::default()
    });
    let cfg = client;

    wifi.set_configuration(&cfg)?;
    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;
    // Print Out Wifi Connection Configuration
    while !wifi.is_connected()? {
        let config = wifi.get_configuration()?;
        println!("Waiting for station {:?}", config);
    }

    log::info!("WiFi initialized!");
    Ok(wifi)
}

pub fn sntp() -> Result<sntp::EspSntp<'static>, Box<dyn Error>> {
    let sntp = sntp::EspSntp::new_default()?;
    log::info!("SNTP initialized, waiting for status!");
    while sntp.get_sync_status() != SyncStatus::Completed {}
    log::info!("SNTP status received!");
    Ok(sntp)
}

pub fn spi(
    spi3: SPI3,
    cs_pin: Gpio10,
    sdo_pin: Gpio11,
    sclk_pin: Gpio12,
    sdi_pin: Gpio13,
) -> Result<MySpiDriver, Box<dyn Error>> {
    let driverconfig = SpiDriverConfig::new();
    let driver = SpiDriver::new(spi3, sclk_pin, sdo_pin, Some(sdi_pin), &driverconfig)?;
    let config = Config::new();
    log::info!("SPI driver initialized!");
    Ok(SpiDeviceDriver::new(driver, Some(cs_pin), &config)?)
}
