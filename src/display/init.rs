use std::error::Error;

use esp_idf_svc::hal::{delay::Ets, gpio::*};

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::{Rgb565, RgbColor},
    text::{Baseline, Text},
    Drawable,
};
use mipidsi::{models::ILI9341Rgb565, Builder, Orientation};

use crate::types::MySpiDriver;

use super::{types::EspDisplayError, DisplayWithBacklight, LARGE_FONT};

pub fn display(
    spi: MySpiDriver,
    rst_pin: Gpio4,
    dc_pin: Gpio5,
    backlight_pin: Gpio6,
) -> Result<DisplayWithBacklight, EspDisplayError> {
    let mut delay = Ets;
    let dc = PinDriver::output(dc_pin)?;
    let rst = PinDriver::output(rst_pin)?;
    let di = SPIInterfaceNoCS::new(spi, dc);
    let mut backlight = PinDriver::output(backlight_pin)?;
    backlight.set_high()?;
    let mut display = Builder::with_model(di, ILI9341Rgb565)
        .init(&mut delay, Some(rst))
        .map_err(|_| Box::<dyn Error>::from("display init"))
        .unwrap();
    log::info!("Display driver initialized!");
    display.clear(Rgb565::BLACK)?;
    match Text::with_baseline("LEAVING IN ", Point::new(1, 0), LARGE_FONT, Baseline::Top)
        .draw(&mut display)
    {
        Ok(_) => (),
        Err(err) => log::error!("display: draw: {:?}", err),
    };
    display.set_orientation(Orientation::Portrait(false))?;
    log::info!("Display fully set up!");

    Ok(DisplayWithBacklight { display, backlight })
}
