use display_interface::DisplayError;
use esp_idf_svc::{
    hal::gpio::{Gpio4, Output, PinDriver},
    sys::EspError,
};
use mipidsi::{models::ILI9341Rgb565, Display};

use crate::types::MySpiInterface;

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
