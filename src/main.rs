use std::thread::sleep;
use std::{error::Error, time::Duration};

use esp_idf_svc::hal::{
    gpio::*,
    peripherals::Peripherals,
    spi::{SpiDeviceDriver, SpiDriver},
};

use display_interface_spi::SPIInterfaceNoCS;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{Rgb666, RgbColor},
};
use mipidsi::models::ILI9341Rgb666;
use mipidsi::Display;

pub mod display;
pub mod init;
use display::DisplayWithBacklight;

fn main() -> Result<(), Box<dyn Error>> {
    init::esp();
    let peri = Peripherals::take()?;
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

fn app(mut display: MySpiDisplay) -> Result<(), Box<dyn Error>> {
    let _ = display.clear(Rgb666::BLUE);

    loop {
        sleep(Duration::from_secs(20));
    }
}

pub type MySpiDriver = SpiDeviceDriver<'static, SpiDriver<'static>>;
pub type MySpiInterface = SPIInterfaceNoCS<MySpiDriver, PinDriver<'static, Gpio5, Output>>;
pub type MySpiDisplay = Display<MySpiInterface, ILI9341Rgb666, PinDriver<'static, Gpio4, Output>>;

