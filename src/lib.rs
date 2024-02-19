//! Rust driver for the [Adafruit I2C LCD backpack](https://www.adafruit.com/product/292) with MCP23008 GPIO expander
//!
//! _NOTE: This library is not made by Adafruit, and is not supported by them. The use of the Adafruit name
//! is for compatibility identification purposes only._
//!
//! ## Overview
//! This crate provides a driver for the Adafruit I2C LCD backpack with MCP23008 GPIO expander. It is designed to be used with the
//! [embedded-hal](https://docs.rs/embedded-hal/latest/embedded_hal/index.html) traits for embeded systems. It supports standard
//! HD44780 based LCD displays.
//!
//! ## Usage
//! To create a new LCD backpack, use the `new` method. This will return a new LCD backpack object. Pass it the type of LCD display you
//! are using, the I2C bus, and the delay object. Both the I2C Bus and Delay objects must implement the relevant embedded-hal traits.
//!
//! ```rust
//! // The embedded-hal traits are used to define the I2C bus and delay objects
//! use embedded_hal::{
//!     blocking::delay::{DelayMs, DelayUs},
//!     blocking::i2c::{Write, WriteRead},
//! };
//! use lcd_backpack::{LcdBackpack, LcdDisplayType};
//!
//! // create the I2C bus per your platform
//! let i2c = ...;
//!
//! // create the delay object per your platform
//! let delay = ...;
//!
//! // create the LCD backpack
//! let mut lcd = LcdBackpack::new(LcdDisplayType::Lcd16x2, i2c, delay);
//!
//! // initialize the LCD
//! if let Err(_e) = lcd.init() {
//!    panic!("Error initializing LCD");
//! }
//! ```
//! This library supports the `core::fmt::Write` trait, allowing it to be used with the `write!` macro. For example:
//! ```rust
//! use core::fmt::Write;
//!
//! // write a string to the LCD
//! if let Err(_e) = write!(lcd, "Hello, world!") {
//!   panic!("Error writing to LCD");
//! }
//! ```
//! The various methods for controlling the LCD are also available. Each returns a `Result` that wraps the LCD backpack object. This
//! allows you to chain the methods together. For example:
//!
//! ```rust
//! // clear the display and home the cursor before writing a string
//! if let Err(_e) = write!(lcd.clear()?.home()?, "Hello, world!") {
//!  panic!("Error writing to LCD");
//! }
//! ```

#![no_std]
#![allow(dead_code, non_camel_case_types, non_upper_case_globals)]
use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    blocking::i2c::{Write, WriteRead},
};
use mcp230xx::{Direction, Level, Mcp23008, Mcp230xx, Register};

const RS_PIN: Mcp23008 = Mcp23008::P1;
const ENABLE_PIN: Mcp23008 = Mcp23008::P2;
const DATA_D4_PIN: Mcp23008 = Mcp23008::P3;
const DATA_D5_PIN: Mcp23008 = Mcp23008::P4;
const DATA_D6_PIN: Mcp23008 = Mcp23008::P5;
const DATA_D7_PIN: Mcp23008 = Mcp23008::P6;
const BACKLIGHT_PIN: Mcp23008 = Mcp23008::P7;

// data pins are in order from least significant bit to most significant bit
const DATA_PINS: [Mcp23008; 4] = [DATA_D4_PIN, DATA_D5_PIN, DATA_D6_PIN, DATA_D7_PIN];

// commands
const LCD_CMD_CLEARDISPLAY: u8 = 0x01; //  Clear display, set cursor position to zero
const LCD_CMD_RETURNHOME: u8 = 0x02; //  Set cursor position to zero
const LCD_CMD_ENTRYMODESET: u8 = 0x04; //  Sets the entry mode
const LCD_CMD_DISPLAYCONTROL: u8 = 0x08; //  Controls the display; does stuff like turning it off and on
const LCD_CMD_CURSORSHIFT: u8 = 0x10; //  Lets you move the cursor
const LCD_CMD_FUNCTIONSET: u8 = 0x20; //  Used to send the function to set to the display
const LCD_CMD_SETCGRAMADDR: u8 = 0x40; //  Used to set the CGRAM (character generator RAM) with characters
const LCD_CMD_SETDDRAMADDR: u8 = 0x80; //  Used to set the DDRAM (Display Data RAM)

// flags for display entry mode
const LCD_FLAG_ENTRYRIGHT: u8 = 0x00; //  Used to set text to flow from right to left
const LCD_FLAG_ENTRYLEFT: u8 = 0x02; //  Uset to set text to flow from left to right
const LCD_FLAG_ENTRYSHIFTINCREMENT: u8 = 0x01; //  Used to 'right justify' text from the cursor
const LCD_FLAG_ENTRYSHIFTDECREMENT: u8 = 0x00; //  Used to 'left justify' text from the cursor

// flags for display on/off control
const LCD_FLAG_DISPLAYON: u8 = 0x04; //  Turns the display on
const LCD_FLAG_DISPLAYOFF: u8 = 0x00; //  Turns the display off
const LCD_FLAG_CURSORON: u8 = 0x02; //  Turns the cursor on
const LCD_FLAG_CURSOROFF: u8 = 0x00; //  Turns the cursor off
const LCD_FLAG_BLINKON: u8 = 0x01; //  Turns on the blinking cursor
const LCD_FLAG_BLINKOFF: u8 = 0x00; //  Turns off the blinking cursor

// flags for display/cursor shift
const LCD_FLAG_DISPLAYMOVE: u8 = 0x08; //  Flag for moving the display
const LCD_FLAG_CURSORMOVE: u8 = 0x00; //  Flag for moving the cursor
const LCD_FLAG_MOVERIGHT: u8 = 0x04; //  Flag for moving right
const LCD_FLAG_MOVELEFT: u8 = 0x00; //  Flag for moving left

// flags for function set
const LCD_FLAG_8BITMODE: u8 = 0x10; //  LCD 8 bit mode
const LCD_FLAG_4BITMODE: u8 = 0x00; //  LCD 4 bit mode
const LCD_FLAG_2LINE: u8 = 0x08; //  LCD 2 line mode
const LCD_FLAG_1LINE: u8 = 0x00; //  LCD 1 line mode
const LCD_FLAG_5x10_DOTS: u8 = 0x04; //  10 pixel high font mode
const LCD_FLAG_5x8_DOTS: u8 = 0x00; //  8 pixel high font mode

/// The type of LCD display. This is used to determine the number of rows and columns, and the row offsets.
pub enum LcdDisplayType {
    /// 20x4 display
    Lcd20x4,
    /// 20x2 display
    Lcd20x2,
    /// 16x2 display
    Lcd16x2,
}

impl LcdDisplayType {
    /// Get the number of rows for the display type
    const fn rows(&self) -> u8 {
        match self {
            LcdDisplayType::Lcd20x4 => 4,
            LcdDisplayType::Lcd20x2 => 2,
            LcdDisplayType::Lcd16x2 => 2,
        }
    }

    /// Get the number of columns for the display type
    const fn cols(&self) -> u8 {
        match self {
            LcdDisplayType::Lcd20x4 => 20,
            LcdDisplayType::Lcd20x2 => 20,
            LcdDisplayType::Lcd16x2 => 16,
        }
    }

    /// Get the row offsets for the display type. This always returns an array of length 4.
    /// For displays with less than 4 rows, the unused rows will be set to offsets offscreen.
    const fn row_offsets(&self) -> [u8; 4] {
        match self {
            LcdDisplayType::Lcd20x4 => [0x00, 0x40, 0x14, 0x54],
            LcdDisplayType::Lcd20x2 => [0x00, 0x40, 0x00, 0x40],
            LcdDisplayType::Lcd16x2 => [0x00, 0x40, 0x10, 0x50],
        }
    }
}

pub struct LcdBackpack<I2C, D> {
    register: Mcp230xx<I2C, Mcp23008>,
    delay: D,
    lcd_type: LcdDisplayType,
    display_function: u8,
    display_control: u8,
    display_mode: u8,
}

/// Errors that can occur when using the LCD backpack
pub enum Error<I2C_ERR> {
    /// I2C error returned from the underlying I2C implementation
    I2cError(I2C_ERR),
    /// The MCP23008 interrupt pin is not found
    InterruptPinError,
    /// Row is out of range
    RowOutOfRange,
    /// Column is out of range
    ColumnOutOfRange,
    /// Formatting error
    #[cfg(feature = "defmt")]
    FormattingError,
}

impl<I2C_ERR> From<I2C_ERR> for Error<I2C_ERR> {
    fn from(err: I2C_ERR) -> Self {
        Error::I2cError(err)
    }
}

impl<I2C_ERR> From<mcp230xx::Error<I2C_ERR>> for Error<I2C_ERR> {
    fn from(err: mcp230xx::Error<I2C_ERR>) -> Self {
        match err {
            mcp230xx::Error::BusError(e) => Error::I2cError(e),
            mcp230xx::Error::InterruptPinError => Error::InterruptPinError,
        }
    }
}

#[cfg(feature = "defmt")]
impl<I2C_ERR> defmt::Format for Error<I2C_ERR>
where
    I2C_ERR: defmt::Format,
{
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            Error::I2cError(e) => defmt::write!(fmt, "I2C error: {:?}", e),
            Error::InterruptPinError => defmt::write!(fmt, "Interrupt pin not found"),
            Error::RowOutOfRange => defmt::write!(fmt, "Row out of range"),
            Error::ColumnOutOfRange => defmt::write!(fmt, "Column out of range"),
            Error::FormattingError => defmt::write!(fmt, "Formatting error"),
        }
    }
}

impl<I2C, I2C_ERR, D> LcdBackpack<I2C, D>
where
    I2C: Write<Error = I2C_ERR> + WriteRead<Error = I2C_ERR>,
    D: DelayMs<u16> + DelayUs<u16>,
{
    /// Create a new LCD backpack with the default I2C address of 0x20
    pub fn new(lcd_type: LcdDisplayType, i2c: I2C, delay: D) -> Self {
        Self::new_with_address(lcd_type, i2c, delay, 0x20)
    }

    /// Create a new LCD backpack with the specified I2C address
    pub fn new_with_address(lcd_type: LcdDisplayType, i2c: I2C, delay: D, address: u8) -> Self {
        let register = match Mcp230xx::<I2C, Mcp23008>::new(i2c, address) {
            Ok(r) => r,
            Err(_) => panic!("Could not create MCP23008"),
        };

        Self {
            register,
            delay,
            lcd_type,
            display_function: LCD_FLAG_4BITMODE | LCD_FLAG_5x8_DOTS | LCD_FLAG_2LINE,
            display_control: LCD_FLAG_DISPLAYON | LCD_FLAG_CURSOROFF | LCD_FLAG_BLINKOFF,
            display_mode: LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT,
        }
    }

    /// Get a mutable reference to the delay object. This is useful as the delay objectis moved into the LCD backpack during initialization.
    pub fn delay(&mut self) -> &mut D {
        &mut self.delay
    }

    /// Initialize the LCD. Must be called before any other methods. Will turn on the blanked display, with no cursor or blinking.
    pub fn init(&mut self) -> Result<&mut Self, Error<I2C_ERR>> {
        // set up back light
        self.register
            .set_direction(BACKLIGHT_PIN, Direction::Output)?;
        self.register.set_gpio(BACKLIGHT_PIN, Level::High)?;

        // set data pins to output
        for pin in DATA_PINS.iter() {
            self.register.set_direction(*pin, Direction::Output)?;
        }

        // RS & Enable piun
        self.register.set_direction(RS_PIN, Direction::Output)?;
        self.register.set_direction(ENABLE_PIN, Direction::Output)?;

        // need to wait 40ms after power rises above 2.7V before sending any commands. wait alittle longer.
        self.delay().delay_ms(50);

        // pull RS & Enable low to start command. RW is hardwired low on backpack.
        self.register.set_gpio(RS_PIN, Level::Low)?;
        self.register.set_gpio(ENABLE_PIN, Level::Low)?;

        // Put LCD into 4 bit mode, device starts in 8 bit mode
        self.write_4_bits(0x03)?;
        self.delay().delay_ms(5);
        self.write_4_bits(0x03)?;
        self.delay().delay_ms(5);
        self.write_4_bits(0x03)?;
        self.delay().delay_us(150);
        self.write_4_bits(0x02)?;

        // set up the display
        self.send_command(LCD_CMD_FUNCTIONSET | self.display_function)?;
        self.send_command(LCD_CMD_DISPLAYCONTROL | self.display_control)?;
        self.send_command(LCD_CMD_ENTRYMODESET | self.display_mode)?;
        self.clear()?;
        self.home()?;

        Ok(self)
    }

    //--------------------------------------------------------------------------------------------------
    // high level commands, for the user!
    //--------------------------------------------------------------------------------------------------

    /// Clear the display
    pub fn clear(&mut self) -> Result<&mut Self, Error<I2C_ERR>> {
        self.send_command(LCD_CMD_CLEARDISPLAY)?;
        self.delay().delay_ms(2);
        Ok(self)
    }

    /// Set the cursor to the home position
    pub fn home(&mut self) -> Result<&mut Self, Error<I2C_ERR>> {
        self.send_command(LCD_CMD_RETURNHOME)?;
        self.delay().delay_ms(2);
        Ok(self)
    }

    /// Set the cursor position at specified column and row
    pub fn set_cursor(&mut self, col: u8, row: u8) -> Result<&mut Self, Error<I2C_ERR>> {
        if row >= self.lcd_type.rows() {
            return Err(Error::RowOutOfRange);
        }
        if col >= self.lcd_type.cols() {
            return Err(Error::ColumnOutOfRange);
        }

        self.send_command(
            LCD_CMD_SETDDRAMADDR | (col + self.lcd_type.row_offsets()[row as usize]),
        )?;
        Ok(self)
    }

    /// Set the cursor visibility
    pub fn show_cursor(&mut self, show_cursor: bool) -> Result<&mut Self, Error<I2C_ERR>> {
        if show_cursor {
            self.display_control |= LCD_FLAG_CURSORON;
        } else {
            self.display_control &= !LCD_FLAG_CURSORON;
        }
        self.send_command(LCD_CMD_DISPLAYCONTROL | self.display_control)?;
        Ok(self)
    }

    /// Set the cursor blinking
    pub fn blink_cursor(&mut self, blink_cursor: bool) -> Result<&mut Self, Error<I2C_ERR>> {
        if blink_cursor {
            self.display_control |= LCD_FLAG_BLINKON;
        } else {
            self.display_control &= !LCD_FLAG_BLINKON;
        }
        self.send_command(LCD_CMD_DISPLAYCONTROL | self.display_control)?;
        Ok(self)
    }

    /// Set the display visibility
    pub fn show_display(&mut self, show_display: bool) -> Result<&mut Self, Error<I2C_ERR>> {
        if show_display {
            self.display_control |= LCD_FLAG_DISPLAYON;
        } else {
            self.display_control &= !LCD_FLAG_DISPLAYON;
        }
        self.send_command(LCD_CMD_DISPLAYCONTROL | self.display_control)?;
        Ok(self)
    }

    /// Scroll the display to the left
    pub fn scroll_display_left(&mut self) -> Result<&mut Self, Error<I2C_ERR>> {
        self.send_command(LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVELEFT)?;
        Ok(self)
    }

    /// Scroll the display to the right
    pub fn scroll_display_right(&mut self) -> Result<&mut Self, Error<I2C_ERR>> {
        self.send_command(LCD_CMD_CURSORSHIFT | LCD_FLAG_DISPLAYMOVE | LCD_FLAG_MOVERIGHT)?;
        Ok(self)
    }

    /// Set the text flow direction to left to right
    pub fn left_to_right(&mut self) -> Result<&mut Self, Error<I2C_ERR>> {
        self.display_mode |= LCD_FLAG_ENTRYLEFT;
        self.send_command(LCD_CMD_ENTRYMODESET | self.display_mode)?;
        Ok(self)
    }

    /// Set the text flow direction to right to left
    pub fn right_to_left(&mut self) -> Result<&mut Self, Error<I2C_ERR>> {
        self.display_mode &= !LCD_FLAG_ENTRYLEFT;
        self.send_command(LCD_CMD_ENTRYMODESET | self.display_mode)?;
        Ok(self)
    }

    /// Set the auto scroll mode
    pub fn autoscroll(&mut self, autoscroll: bool) -> Result<&mut Self, Error<I2C_ERR>> {
        if autoscroll {
            self.display_mode |= LCD_FLAG_ENTRYSHIFTINCREMENT;
        } else {
            self.display_mode &= !LCD_FLAG_ENTRYSHIFTINCREMENT;
        }
        self.send_command(LCD_CMD_ENTRYMODESET | self.display_mode)?;
        Ok(self)
    }

    /// Create a new custom character
    pub fn create_char(
        &mut self,
        location: u8,
        charmap: [u8; 8],
    ) -> Result<&mut Self, Error<I2C_ERR>> {
        self.send_command(LCD_CMD_SETCGRAMADDR | ((location & 0x7) << 3))?;
        for &charmap_byte in charmap.iter() {
            self.write_data(charmap_byte)?;
        }
        Ok(self)
    }

    /// Prints a string to the LCD at the current cursor position
    pub fn print(&mut self, text: &str) -> Result<&mut Self, Error<I2C_ERR>> {
        for c in text.chars() {
            self.write_data(c as u8)?;
        }
        Ok(self)
    }

    //--------------------------------------------------------------------------------------------------
    // Internal data writing functions
    //--------------------------------------------------------------------------------------------------

    /// Write 4 bits to the LCD
    fn write_4_bits(&mut self, value: u8) -> Result<(), Error<I2C_ERR>> {
        // get the current value of the register byte
        let mut register_contents = self.register.read(Register::GPIO.into())?;

        // set bit 0, data pin 4
        for (index, pin) in DATA_PINS.iter().enumerate() {
            let bit_mask = 1 << (*pin as u8);
            register_contents &= !bit_mask;
            if value & (1 << index) != 0 {
                register_contents |= bit_mask;
            }
        }

        // set the enable pin low in the register_contents
        register_contents &= !(1 << (ENABLE_PIN as u8));

        // write the new register contents
        self.register
            .write(Register::GPIO.into(), register_contents)?;

        // pulse ENABLE pin quickly using the known value of the register contents
        self.delay().delay_us(1);
        register_contents |= 1 << (ENABLE_PIN as u8); // set enable pin high
        self.register
            .write(Register::GPIO.into(), register_contents)?;
        self.delay().delay_us(1);
        register_contents &= !(1 << (ENABLE_PIN as u8)); // set enable pin low
        self.register
            .write(Register::GPIO.into(), register_contents)?;
        self.delay().delay_us(100);

        Ok(())
    }

    /// Write 8 bits to the LCD using 4 bit mode
    fn write_8_bits(&mut self, value: u8) -> Result<(), Error<I2C_ERR>> {
        self.write_4_bits(value >> 4)?;
        self.write_4_bits(value & 0x0F)?;
        Ok(())
    }

    /// Send a command to the LCD
    pub fn send_command(&mut self, command: u8) -> Result<(), Error<I2C_ERR>> {
        self.register.set_gpio(RS_PIN, Level::Low)?;
        self.write_8_bits(command)?;
        Ok(())
    }

    /// Send data to the LCD
    pub fn write_data(&mut self, value: u8) -> Result<(), Error<I2C_ERR>> {
        self.register.set_gpio(RS_PIN, Level::High)?;
        self.write_8_bits(value)?;
        Ok(())
    }

    /// Pulse the enable pin
    fn pulse_enable(&mut self) -> Result<(), Error<I2C_ERR>> {
        self.register.set_gpio(ENABLE_PIN, Level::Low)?;
        self.delay().delay_us(1);
        self.register.set_gpio(ENABLE_PIN, Level::High)?;
        self.delay().delay_us(1);
        self.register.set_gpio(ENABLE_PIN, Level::Low)?;
        self.delay().delay_us(100);

        Ok(())
    }
}

/// Implement the `core::fmt::Write` trait for the LCD backpack, allowing it to be used with the `write!` macro.
impl<I2C, I2C_ERR, D> core::fmt::Write for LcdBackpack<I2C, D>
where
    I2C: Write<Error = I2C_ERR> + WriteRead<Error = I2C_ERR>,
    D: DelayMs<u16> + DelayUs<u16>,
{
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        if let Err(_error) = self.print(s) {
            return Err(core::fmt::Error);
        }
        Ok(())
    }
}
