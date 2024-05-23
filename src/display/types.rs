use display_interface::DisplayError;
use embedded_graphics::{
    geometry::Point,
    mono_font::MonoTextStyle,
    pixelcolor::Rgb565,
    prelude::*,
    text::{renderer::CharacterStyle, Baseline, Text},
};
use embedded_vintage_fonts::FONT_24X32;
use esp_idf_svc::{
    hal::gpio::{Gpio4, Output, PinDriver},
    sys::EspError,
};
use mipidsi::{models::ILI9341Rgb565, Display};

use crate::{client::types::Departure, types::MySpiInterface};

pub type MySpiDisplay = Display<MySpiInterface, ILI9341Rgb565, PinDriver<'static, Gpio4, Output>>;

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

#[derive(Clone, Copy)]
pub struct TimeCounter {
    y_start: i32,
    height: i32,
}

impl TimeCounter {
    pub fn new(y_start: i32, height: i32) -> TimeCounter {
        TimeCounter { y_start, height }
    }

    pub fn draw(
        &self,
        display: &mut MySpiDisplay,
        departure: Departure,
        line: i32,
    ) -> Result<(), EspDisplayError> {
        // let (prev_minutes, prev_seconds) = prev_departure.format_time();
        let (minutes, seconds) = departure.format_time();

        let leaving = format!("{}:{:02}", minutes, seconds);
        let formatted_time = format!("{: >5}", leaving);

        let mut style = MonoTextStyle::new(&FONT_24X32, Rgb565::WHITE);

        style.set_background_color(Some(Rgb565::BLACK));

        let t = Text::with_baseline(
            formatted_time.as_str(),
            Point::new(72, self.y_start + line * self.height + (self.height / 2)),
            style,
            Baseline::Middle,
        );

        match t.draw(display) {
            Ok(_) => (),
            Err(err) => log::error!("display: draw: {:?}", err),
        };

        Ok(())
    }
}
