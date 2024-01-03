#![no_std]
#![allow(dead_code, non_camel_case_types, non_upper_case_globals)]

extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
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
const LCD_FLAG_ENTRYRIGHT: u8 = 0x00;//  Used to set text to flow from right to left
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

pub struct LcdBackpack<I2C, D> {
    register: Mcp230xx<I2C, Mcp23008>,
    delay: Rc<RefCell<D>>,
    display_function: u8,
}

/// Errors that can occur when using the LCD backpack
pub enum Error<I2C_ERR> {
    /// I2C error returned from the underlying I2C implementation
    I2cError(I2C_ERR),
    /// The MCP23008 interrupt pin is not found
    InterruptPinError,
    /// Row is out of range
    RowOutOfRange,
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

impl<I2C, I2C_ERR, D> LcdBackpack<I2C, D>
where
    I2C: Write<Error = I2C_ERR> + WriteRead<Error = I2C_ERR>,
    D: DelayMs<u16> + DelayUs<u16>,
{
    pub fn new(i2c: &Rc<RefCell<I2C>>, delay: &Rc<RefCell<D>>) -> Self {
        Self::new_with_address(i2c, delay, 0x20)
    }

    pub fn new_with_address(i2c: &Rc<RefCell<I2C>>, delay: &Rc<RefCell<D>>, address: u8) -> Self {
        let register = match Mcp230xx::<I2C, Mcp23008>::new(i2c, address) {
            Ok(r) => r,
            Err(_) => panic!("Could not create MCP23008"),
        };

        Self {
            register,
            delay: delay.clone(),
            display_function: LCD_FLAG_4BITMODE | LCD_FLAG_5x8_DOTS | LCD_FLAG_2LINE,
        }
    }

    /// Initialize the LCD. Must be called before any other methods. Will turn on the blanked display, with no cursor or blinking.
    pub fn init(&mut self) -> Result<(), Error<I2C_ERR>> {
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
        self.delay.borrow_mut().delay_ms(50);

        // pull RS & Enable low to start command. RW is hardwired low on backpack.
        self.register.set_gpio(RS_PIN, Level::Low)?;
        self.register.set_gpio(ENABLE_PIN, Level::Low)?;

        // Put LCD into 4 bit mode, device starts in 8 bit mode
        self.write_4_bits(0x03)?;
        self.delay.borrow_mut().delay_ms(5);
        self.write_4_bits(0x03)?;
        self.delay.borrow_mut().delay_ms(5);
        self.write_4_bits(0x03)?;
        self.delay.borrow_mut().delay_us(150);
        self.write_4_bits(0x02)?;

        self.send_command(LCD_CMD_FUNCTIONSET | self.display_function)?;

        self.send_command(LCD_CMD_DISPLAYCONTROL |  LCD_FLAG_DISPLAYON | LCD_FLAG_CURSORON | LCD_FLAG_BLINKOFF)?;

        self.clear_display()?;

        self.send_command(LCD_CMD_ENTRYMODESET | LCD_FLAG_ENTRYLEFT | LCD_FLAG_ENTRYSHIFTDECREMENT)?;

        self.home()?;

        Ok(())
    }

    pub fn clear_display(&mut self) -> Result<(), Error<I2C_ERR>> {
        self.send_command(LCD_CMD_CLEARDISPLAY)?;
        self.delay.borrow_mut().delay_ms(2);
        Ok(())
    }

    pub fn home(&mut self) -> Result<(), Error<I2C_ERR>> {
        self.send_command(LCD_CMD_RETURNHOME)?;
        self.delay.borrow_mut().delay_ms(2);
        Ok(())
    }

    pub fn set_cursor(&mut self, col: u8, row: u8) -> Result<(), Error<I2C_ERR>> {
        let row_offsets = [0x00, 0x40, 0x10, 0x50]; // TODO: make this configurable
        if row > row_offsets.len() as u8 {
            return Err(Error::RowOutOfRange);
        }
        self.send_command(LCD_CMD_SETDDRAMADDR | (col + row_offsets[row as usize]))?;
        Ok(())
    }

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
        self.delay.borrow_mut().delay_us(1);
        register_contents |= 1 << (ENABLE_PIN as u8); // set enable pin high
        self.register
            .write(Register::GPIO.into(), register_contents)?;
        self.delay.borrow_mut().delay_us(1);
        register_contents &= !(1 << (ENABLE_PIN as u8)); // set enable pin low
        self.register
            .write(Register::GPIO.into(), register_contents)?;
        self.delay.borrow_mut().delay_us(100);

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

    fn pulse_enable(&mut self) -> Result<(), Error<I2C_ERR>> {
        self.register.set_gpio(ENABLE_PIN, Level::Low)?;
        self.delay.borrow_mut().delay_us(1);
        self.register.set_gpio(ENABLE_PIN, Level::High)?;
        self.delay.borrow_mut().delay_us(1);
        self.register.set_gpio(ENABLE_PIN, Level::Low)?;
        self.delay.borrow_mut().delay_us(100);

        Ok(())
    }

    // ------------------ high level commands ---------------------

    pub fn print(&mut self, text: &str) -> Result<(), Error<I2C_ERR>> {
        for c in text.chars() {
            self.write_data(c as u8)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
