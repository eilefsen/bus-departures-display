use esp_idf_svc::hal::gpio::*;
use mipidsi::models::ILI9341Rgb666;

use crate::MySpiDisplay;

pub mod init;
pub struct DisplayWithBacklight {
    pub display: MySpiDisplay,
    pub backlight: PinDriver<'static, Gpio6, Output>,
}
