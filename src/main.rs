use std::thread::sleep;
use std::{error::Error, time::Duration};

use embedded_graphics::primitives::Rectangle;
use embedded_vintage_fonts::{FONT_12X16, FONT_24X32};
use esp_idf_svc::hal::{
    gpio::*,
    peripherals::Peripherals,
    spi::{SpiDeviceDriver, SpiDriver},
};

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::{Baseline, Text};
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb666, RgbColor},
    prelude::*,
};
use mipidsi::models::ILI9341Rgb666;
use mipidsi::Display;

pub mod client;
pub mod display;
pub mod init;
use client::{util::make_query, EnturClient};
use display::DisplayWithBacklight;

use crate::client::types::Departure;

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

const LARGE_FONT: MonoTextStyle<'_, Rgb666> = MonoTextStyle::new(&FONT_24X32, Rgb666::WHITE);
const NORMAL_FONT: MonoTextStyle<'_, Rgb666> = MonoTextStyle::new(&FONT_12X16, Rgb666::WHITE);

fn main() -> Result<(), Box<dyn Error>> {
    init::esp();
    let peri = Peripherals::take()?;
    let _wifi = match init::wifi(peri.modem) {
        Ok(w) => {
            log::info!("WiFi Successfully Connected!");
            w
        }
        Err(err) => {
            log::error!("Could not connect to WiFi!");
            return Err(err);
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

    // let data = client.read_write_request()?;

    app(display)
}

fn app(mut display: MySpiDisplay) -> Result<(), Box<dyn Error>> {
    display
        .set_orientation(mipidsi::Orientation::Portrait(false))
        .unwrap();
    let _ = display.clear(Rgb666::BLACK);

    match Text::with_baseline("LEAVING IN ", Point::new(1, 0), LARGE_FONT, Baseline::Top)
        .draw(&mut display)
    {
        Ok(_) => (),
        Err(err) => log::error!("display: draw: {:?}", err),
    };
    log::info!("Display initialized!");
    log::info!("Initialization complete!");

    // Start application
    const SLEEP_SECONDS: u64 = 20;
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
    let client = client::EnturClient::new(url, headers, query)?;
    loop {
        let data = match client.read_write_request() {
            Ok(val) => val,
            Err(err) => {
                log::error!("{}", err);
                log::info!("Sleeping for {} Seconds", SLEEP_SECONDS);
                sleep(Duration::from_secs(SLEEP_SECONDS));
                continue;
            }
        };
        for _ in 0..(SLEEP_SECONDS / 4) {
            let departures = Departure::from_top_level_data(data.clone());
            log::info!("Response json: {:?}", departures);
            match display_departures(departures, &mut display) {
                Ok(x) => x,
                Err(err) => log::error!("{}", err),
            };
            sleep(Duration::from_secs(SLEEP_SECONDS / 4))
        }
    }
}

fn display_departures(
    departures: Vec<Departure>,
    display: &mut MySpiDisplay,
) -> Result<(), Box<dyn Error>> {
    display
        .fill_solid(
            &Rectangle::new(Point::new(0, 32), Size::new(240, 288)),
            Rgb666::BLACK,
        )
        .unwrap();

    for (i, d) in departures.iter().enumerate() {
        {
            display
                .fill_solid(
                    &Rectangle::new(Point::new(12, (i as i32 + 1) * 36), Size::new(50, 32)),
                    Rgb666::BLUE,
                )
                .unwrap();
            match Text::with_baseline(
                d.line_number.as_str(),
                Point::new(16, (i as i32 + 1) * 32),
                LARGE_FONT,
                Baseline::Top,
            )
            .draw(display)
            {
                Ok(_) => (),
                Err(err) => log::error!("display: draw: {:?}", err),
            };
        }
        match Text::with_baseline(
            format!("{: >5}", d.leaving_in).as_str(),
            Point::new(72, (i as i32 + 1) * 32),
            LARGE_FONT,
            Baseline::Top,
        )
        .draw(display)
        {
            Ok(_) => (),
            Err(err) => log::error!("display: draw: {:?}", err),
        };
    }

    Ok(())
}

pub type MySpiDriver = SpiDeviceDriver<'static, SpiDriver<'static>>;
pub type MySpiInterface = SPIInterfaceNoCS<MySpiDriver, PinDriver<'static, Gpio5, Output>>;
pub type MySpiDisplay = Display<MySpiInterface, ILI9341Rgb666, PinDriver<'static, Gpio4, Output>>;
