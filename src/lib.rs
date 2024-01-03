#![no_std]
extern crate alloc;

use alloc::rc::Rc;
use core::cell::RefCell;
use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    blocking::i2c::{Write, WriteRead},
};
use mcp230xx::{Mcp230xx, Mcp23008};




pub struct LcdBackpack<I2C, D> {
    register: Mcp230xx<I2C, Mcp23008>,
    delay: Rc<RefCell<D>>,
    address: u8,
}

impl<I2C, D> LcdBackpack<I2C, D>
where
    I2C: Write<Error = ()> + WriteRead<Error = ()>,
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
            address,
        }
    }

    pub fn init(&mut self) {

    }

}

#[cfg(test)]
mod tests {
    use super::*;


}
