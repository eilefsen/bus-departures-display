use std::error::Error;

use display_interface_spi::SPIInterfaceNoCS;
use esp_idf_svc::hal::{delay::Ets, gpio::*};
use mipidsi::{models::ILI9341Rgb666, Builder};

use crate::MySpiDriver;

use super::DisplayWithBacklight;

pub fn display(
    spi: MySpiDriver,
    rst_pin: Gpio4,
    dc_pin: Gpio5,
    backlight_pin: Gpio6,
) -> Result<DisplayWithBacklight, Box<dyn Error>> {
    let mut delay = Ets;
    let dc = PinDriver::output(dc_pin)?;
    let rst = PinDriver::output(rst_pin)?;
    let di = SPIInterfaceNoCS::new(spi, dc);
    let mut backlight = PinDriver::output(backlight_pin)?;
    backlight.set_high()?;
    let display = Builder::with_model(di, ILI9341Rgb666)
        .init(&mut delay, Some(rst))
        .map_err(|_| Box::<dyn Error>::from("display init"))?;
    log::info!("Display driver initialized!");
    Ok(DisplayWithBacklight { display, backlight })
}
