#![no_std]
#![no_main]

#[macro_use] extern crate log;
#[macro_use] extern crate alloc;

use core::sync::atomic::{self, AtomicBool};

use embedded_graphics::geometry::{Point, Size};
use embedded_graphics::mono_font::ascii::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::{Rgb565, RgbColor};
use embedded_graphics::primitives::{Primitive, PrimitiveStyle, Rectangle};
use embedded_graphics::text::Text;
use esp_idf_svc::hal::gpio::{self, AnyInputPin, InterruptType, PinDriver, Pull};
use esp_idf_svc::hal::prelude::FromValueType;
use esp_idf_svc::hal::spi::{SpiConfig, SpiDeviceDriver, SpiDriver, SpiDriverConfig};
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::sys::EspError;
use st7735_lcd::ST7735;
use embedded_graphics::{draw_target::DrawTarget, Drawable};

struct Button<'d, Pin: gpio::Pin> {
  pin: PinDriver<'d, Pin, gpio::Input>,
  pressed: AtomicBool,
}

impl<'d, Pin: gpio::InputPin + gpio::OutputPin> Button<'d, Pin> {
  fn new(pin: PinDriver<'d, Pin, gpio::Input>) -> Self {
    Self { pin, pressed: AtomicBool::new(false) }
  }

  fn set_interrupt(&mut self) -> Result<(), EspError> {
    self.pin.set_pull(Pull::Up)?;
    self.pin.set_interrupt_type(InterruptType::NegEdge)?;
    unsafe {
      let pressed = &*(&self.pressed as *const AtomicBool);
      self.pin.subscribe_nonstatic(move || pressed.store(true, atomic::Ordering::Relaxed))?;
    }
    self.pin.enable_interrupt()?;
    Ok(())
  }

  fn is_pressed(&mut self) -> bool {
    let b = self.pressed.load(atomic::Ordering::Relaxed);
    if b {
      self.pressed.store(false, atomic::Ordering::Relaxed);
      self.pin.enable_interrupt().ok();
    }
    b
  }
}

#[no_mangle]
fn main() {
  // It is necessary to call this function once. Otherwise some patches to the runtime
  // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
  esp_idf_svc::sys::link_patches();
  // Bind the log crate to the ESP Logging facilities
  esp_idf_svc::log::EspLogger::initialize_default();

  let peripherals = Peripherals::take().unwrap();
  let mut delay = esp_idf_svc::hal::delay::Delay::new_default();
  log::info!("Hello, world!");

  let k1 = PinDriver::input(peripherals.pins.gpio8).unwrap();
  let k2 = PinDriver::input(peripherals.pins.gpio10).unwrap();
  let mut button_k1 = Button::new(k1);
  let mut button_k2 = Button::new(k2);
  button_k1.set_interrupt().unwrap();
  button_k2.set_interrupt().unwrap();

  // TFT_MOSI = 4
  // TFT_SCLK = 3
  // TFT_CS = 2
  // TFT_DC = 0
  // TFT_RST = 5
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

  let bg_style = PrimitiveStyle::with_fill(Rgb565::BLUE);
  let mut counter = 0;
  loop {
    info!("loop...");
    let time = esp_idf_svc::systime::EspSystemTime.now();
    Rectangle::new(Point::new(0, 21), Size::new(128, 20))
      .into_styled(bg_style)
      .draw(&mut display)
      .unwrap();
    Text::new(&format!("time: {}", time.as_secs()), Point::new(10, 40), style)
      .draw(&mut display)
      .unwrap();

    if button_k1.is_pressed() {
      info!("k1 pressed");
      counter += 1;
    }
    if button_k2.is_pressed() {
      info!("k2 pressed");
      counter = 0;
    }
    Rectangle::new(Point::new(0, 41), Size::new(128, 20))
      .into_styled(bg_style)
      .draw(&mut display)
      .unwrap();
    Text::new(&format!("counter: {}", counter), Point::new(10, 60), style)
      .draw(&mut display)
      .unwrap();
    delay.delay_ms(200);
  }
}
