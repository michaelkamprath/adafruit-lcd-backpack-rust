#![no_std]
#![no_main]
use adafruit_lcd_backpack::{Error, LcdBackpack, LcdDisplayType};
use core::fmt::Write;
use defmt::{error, panic};
use defmt_rtt as _;
use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    blocking::i2c,
};
use panic_probe as _;
use rp_pico::entry;
use rp_pico::hal::{fugit::HertzU32, gpio, prelude::*};

#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = rp_pico::hal::pac::Peripherals::take().unwrap();
    let core = rp_pico::hal::pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = rp_pico::hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    // The default is to generate a 125 MHz system clock
    let clocks = rp_pico::hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // The single-cycle I/O block controls our GPIO pins
    let sio = rp_pico::hal::Sio::new(pac.SIO);

    // Set the pins up according to their function on this particular board
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // The delay object lets us wait for specified amounts of time. Wrap it in a
    // RefCell so we can share it between the main loop and other functions.
    let delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // set up I2C. Also wrap it in a RefCell so we can share it between the main loop and other functions.
    let i2c = rp_pico::hal::I2C::new_controller(
        pac.I2C0,
        pins.gpio4.into_function::<gpio::FunctionI2c>(),
        pins.gpio5.into_function::<gpio::FunctionI2c>(),
        HertzU32::from_raw(400_000),
        &mut pac.RESETS,
        clocks.system_clock.freq(),
    );

    // create the LEA backpack object
    let mut lcd_backpack = LcdBackpack::new(LcdDisplayType::Lcd16x2, i2c, delay);
    if let Err(e) = lcd_backpack.init() {
        panic!("Error initializing LCD: {}", e);
    }

    loop {
        if let Err(e) = write_lcd_sequence(&mut lcd_backpack) {
            error!("Error writing to LCD: {}", e);
        }
    }
}

#[allow(non_camel_case_types)]
fn write_lcd_sequence<TWI, TWI_ERR, DELAY>(
    lcd: &mut LcdBackpack<TWI, DELAY>,
) -> Result<(), Error<TWI_ERR>>
where
    TWI: i2c::Write<Error = TWI_ERR> + i2c::WriteRead<Error = TWI_ERR>,
    DELAY: DelayMs<u16> + DelayUs<u16> + DelayMs<u8>,
{
    // clear the display;
    if let Err(core::fmt::Error) = write!(lcd.clear()?.home()?, "Hello, world!") {
        error!("Error writing to LCD");
    }
    // wait 1 second
    lcd.delay().delay_ms(2000u16);
    // clear the display
    if let Err(core::fmt::Error) = write!(lcd.set_cursor(0, 1)?, "I'm LCD Backpack") {
        error!("Error writing to LCD");
    }
    // wait 1 second
    lcd.delay().delay_ms(2000u16);
    // clear the display
    lcd.clear()?;
    lcd.delay().delay_ms(500u16);

    Ok(())
}
