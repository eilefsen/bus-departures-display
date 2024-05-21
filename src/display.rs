use display_interface::DisplayError;
use esp_idf_svc::{hal::gpio::*, sys::EspError};

use crate::MySpiDisplay;

pub mod init;
pub struct DisplayWithBacklight {
    pub display: MySpiDisplay,
    pub backlight: PinDriver<'static, Gpio6, Output>,
}

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
