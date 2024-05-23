use std::{
    sync::{Arc, Mutex},
    thread::{self, sleep},
};

use client::SLEEP_SECONDS;

use esp_idf_svc::hal::peripherals::Peripherals;

pub mod client;
pub mod display;
pub mod init;
pub mod types;
use client::util::make_query;
use display::{types::MySpiDisplay, DisplayWithBacklight, EspDisplayError};

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
    #[default("")]
    from_place1: &'static str,
    #[default("")]
    to_place1: &'static str,
    #[default("")]
    from_place2: &'static str,
    #[default("")]
    to_place2: &'static str,
}

fn main() -> Result<(), EspDisplayError> {
    init::esp();

    let peri = Peripherals::take()?;
    let _wifi = match init::wifi(peri.modem) {
        Ok(w) => {
            log::info!("WiFi Successfully Connected!");
            w
        }
        Err(err) => {
            log::error!("Could not connect to WiFi!");
            return Err(err.into());
        }
    };
    let _sntp = init::sntp()?;

    let spi = init::spi(
        peri.spi3,
        peri.pins.gpio10,
        peri.pins.gpio11,
        peri.pins.gpio12,
        peri.pins.gpio13,
    )?;
    let DisplayWithBacklight {
        display,
        backlight: _backlight,
    } = init::display(spi, peri.pins.gpio4, peri.pins.gpio5, peri.pins.gpio6)?;

    log::info!("Initialization complete!");
    app(display)
}

fn app(mut display: MySpiDisplay) -> Result<(), EspDisplayError> {
    // Start application
    let url = "https://api.entur.io/journey-planner/v3/graphql";
    let headers = vec![
        ("content-type", "application/json"),
        ("ET-Client-Name", "eilefsen-entur_display"),
    ];
    let query = format!(
        r#"{{
			trip1: {}, trip2: {}
		}}"#,
        make_query(CONFIG.from_place1, CONFIG.to_place1),
        make_query(CONFIG.from_place2, CONFIG.to_place2),
    );

    let client = client::EnturClient::new(url, headers, query).unwrap();

    //initialize shared_data with http data
    let shared_data = Arc::new(Mutex::new(client.read_write_request().unwrap()));
    let display_data = Arc::clone(&shared_data);

    let _display_thread = thread::Builder::new().stack_size(8192).spawn(move || loop {
        let data = display_data.lock().unwrap().clone();
        display::display_loop(data, &mut display)
    });

    loop {
        // sleep first, since we initialized with a similar request earlier (avoid rate limit)
        sleep(std::time::Duration::from_secs(SLEEP_SECONDS));
        let data = match client.read_write_request() {
            Ok(val) => val,
            Err(err) => {
                log::error!("{}", err);
                log::info!("Sleeping for {} Seconds", SLEEP_SECONDS);
                sleep(std::time::Duration::from_secs(SLEEP_SECONDS));
                continue;
            }
        };
        let mut d = shared_data.lock().unwrap();
        *d = data;
    }
}
