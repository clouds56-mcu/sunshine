#![no_std]
#![no_main]

#[macro_use]
extern crate log;

use embedded_graphics::geometry::Point;
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::text::Text;
use esp_idf_svc::hal::gpio::{AnyInputPin, PinDriver};
use esp_idf_svc::hal::prelude::FromValueType;
use esp_idf_svc::hal::spi::{SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_svc::hal::peripherals::Peripherals;
use st7735_lcd::ST7735;
use embedded_graphics::{draw_target::DrawTarget, Drawable};

#[no_mangle]
fn main() {
  // It is necessary to call this function once. Otherwise some patches to the runtime
  // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
  esp_idf_svc::sys::link_patches();
  // Bind the log crate to the ESP Logging facilities
  esp_idf_svc::log::EspLogger::initialize_default();

  let mut delay = esp_idf_svc::hal::delay::Delay::new_default();
  log::info!("Hello, world!");
  // TFT_MOSI = 4
  // TFT_SCLK = 3
  // TFT_CS = 2
  // TFT_DC = 0
  // TFT_RST = 5
  let peripherals = Peripherals::take().unwrap();
  let spi_driver = SpiDriver::new(
    peripherals.spi2,
    peripherals.pins.gpio3,
    peripherals.pins.gpio4,
    None::<AnyInputPin>,
    &SpiDriverConfig::default(),
  ).unwrap();
  let spi = SpiDeviceDriver::new(
    spi_driver,
    Some(peripherals.pins.gpio2),
    &SpiConfig::default().baudrate(40.MHz().into()),
  ).unwrap();
  let dc = PinDriver::output(peripherals.pins.gpio0).unwrap();
  let rst = PinDriver::output(peripherals.pins.gpio5).unwrap();
  let mut display = ST7735::new(spi, dc, Some(rst), true, true, 128, 128);
  display.set_offset(2, 1);

  display.init(&mut delay).unwrap();
  display.clear(Rgb565::BLUE).unwrap();
  let style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE);
  Text::new("Hello, world!", Point::new(10, 20), style)
    .draw(&mut display)
    .unwrap();

  loop {
    info!("loop...");
    delay.delay_ms(1000);
  }
}
