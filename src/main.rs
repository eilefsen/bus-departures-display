use std::thread::sleep;
use std::time::{Duration, Instant};

use embedded_graphics::pixelcolor::Rgb565;
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
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::RgbColor, prelude::*};
use mipidsi::Display;
use mipidsi::{models::ILI9341Rgb565, Orientation};

pub mod client;
pub mod display;
pub mod init;
use client::util::make_query;
use display::{DisplayWithBacklight, EspDisplayError};

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

const LARGE_FONT: MonoTextStyle<'_, Rgb565> = MonoTextStyle::new(&FONT_24X32, Rgb565::WHITE);
const LARGE_FONT_BLACK: MonoTextStyle<'_, Rgb565> = MonoTextStyle::new(&FONT_24X32, Rgb565::BLACK);
const _NORMAL_FONT: MonoTextStyle<'_, Rgb565> = MonoTextStyle::new(&FONT_12X16, Rgb565::WHITE);

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

    app(display)
}

fn app(mut display: MySpiDisplay) -> Result<(), EspDisplayError> {
    display.set_orientation(Orientation::Portrait(false))?;

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
    let mut departures: Vec<Departure> = vec![];
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

        let y_offset = 4;
        let y_gap = 4;
        let height = 36 + y_gap;

        // erase previous times
        display_departure_times(
            &departures,
            &mut display,
            LARGE_FONT_BLACK,
            y_offset,
            height,
        )?;

        departures = Departure::from_top_level_data(data.clone());

        display_departure_lines(&departures, &mut display, y_offset, height)?;

        for _ in 0..SLEEP_SECONDS {
            display_departure_times(
                &departures,
                &mut display,
                LARGE_FONT_BLACK,
                y_offset,
                height,
            )?;
            departures = Departure::from_top_level_data(data.clone());
            log::info!("Response json: {:?}", departures);
            // erase previous times
            display_departure_times(&departures, &mut display, LARGE_FONT, y_offset, height)?;
            sleep(Duration::from_secs(1))
        }
    }
}

fn display_seconds() {}

fn display_departure_times(
    departures: &[Departure],
    display: &mut MySpiDisplay,
    text_style: MonoTextStyle<'_, Rgb565>,
    y_offset: i32,
    height: i32,
) -> Result<(), EspDisplayError> {
    for (i, d) in departures.iter().enumerate() {
        let formatted_time = format!("{: >5}", d.leaving_in);
        let t = Text::with_baseline(
            formatted_time.as_str(),
            Point::new(72, y_offset + ((i as i32 + 1) * height + (height / 2))),
            text_style,
            Baseline::Middle,
        );
        match t.draw(display) {
            Ok(_) => (),
            Err(err) => log::error!("display: draw: {:?}", err),
        };
    }

    Ok(())
}

fn display_departure_lines(
    departures: &[Departure],
    display: &mut MySpiDisplay,
    y_offset: i32,
    height: i32,
) -> Result<(), EspDisplayError> {
    for (i, d) in departures.iter().enumerate() {
        display.fill_solid(
            &Rectangle::new(
                Point::new(12, y_offset + ((i as i32 + 1) * height)),
                Size::new(50, 36),
            ),
            Rgb565::RED,
        )?;
        let t = Text::with_baseline(
            d.line_number.as_str(),
            Point::new(14, y_offset + ((i as i32 + 1) * height + (height / 2))),
            LARGE_FONT,
            Baseline::Middle,
        );
        match t.draw(display) {
            Ok(_) => (),
            Err(err) => log::error!("display: draw: {:?}", err),
        };
    }

    Ok(())
}

pub type MySpiDriver = SpiDeviceDriver<'static, SpiDriver<'static>>;
pub type MySpiInterface = SPIInterfaceNoCS<MySpiDriver, PinDriver<'static, Gpio5, Output>>;
pub type MySpiDisplay = Display<MySpiInterface, ILI9341Rgb565, PinDriver<'static, Gpio4, Output>>;
