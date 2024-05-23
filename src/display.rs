use std::thread::sleep;

use display_interface::DisplayError;
use embedded_graphics::Drawable;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    mono_font::MonoTextStyle,
    pixelcolor::{Rgb565, RgbColor},
    primitives::Rectangle,
    text::{Baseline, Text},
};
use embedded_vintage_fonts::FONT_24X32;
use esp_idf_svc::{hal::gpio::*, sys::EspError};

use crate::client::{
    types::{Departure, TopLevelData},
    SLEEP_SECONDS,
};

use self::types::{MySpiDisplay, TimeCounter};

pub mod init;
pub mod types;
pub struct DisplayWithBacklight {
    pub display: MySpiDisplay,
    pub backlight: PinDriver<'static, Gpio6, Output>,
}

pub const LARGE_FONT: MonoTextStyle<'_, Rgb565> = MonoTextStyle::new(&FONT_24X32, Rgb565::WHITE);

#[derive(Debug)]
pub enum EspDisplayError {
    EspError(EspError),
    DisplayError(DisplayError),
}

impl From<EspError> for EspDisplayError {
    fn from(err: EspError) -> Self {
        EspDisplayError::EspError(err)
    }
}

impl From<DisplayError> for EspDisplayError {
    fn from(err: DisplayError) -> Self {
        EspDisplayError::DisplayError(err)
    }
}

pub fn display_loop(data: TopLevelData, display: &mut MySpiDisplay) {
    let y_offset = 4;
    let y_gap = 4;
    let height = 36 + y_gap;
    let tc = TimeCounter::new(y_offset, height);
    let departures = Departure::from_top_level_data(data.clone());
    display_departure_lines(&departures, display, y_offset, height).unwrap();
    for _ in 0..SLEEP_SECONDS {
        let departures = Departure::from_top_level_data(data.clone());
        display_departure_times(&departures, display, tc).unwrap();
        log::info!("Response json: {:?}", departures);
        sleep(std::time::Duration::from_secs(1))
    }
}

fn display_departure_times(
    departures: &[Departure],
    display: &mut MySpiDisplay,
    tc: TimeCounter,
) -> Result<(), EspDisplayError> {
    for (i, d) in departures.iter().enumerate() {
        let i_i32 = i32::try_from(i).unwrap();

        tc.draw(display, d.clone(), i_i32 + 1)?;
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
