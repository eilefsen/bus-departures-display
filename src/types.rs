use display_interface_spi::SPIInterfaceNoCS;
use esp_idf_svc::hal::{
    gpio::{Gpio5, Output, PinDriver},
    spi::{SpiDeviceDriver, SpiDriver},
};

pub type MySpiDriver = SpiDeviceDriver<'static, SpiDriver<'static>>;
pub type MySpiInterface = SPIInterfaceNoCS<MySpiDriver, PinDriver<'static, Gpio5, Output>>;
