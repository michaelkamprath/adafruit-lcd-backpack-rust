# adafruit-lcd-backpack

<!-- cargo-sync-readme start -->

Rust driver for the [Adafruit I2C LCD backpack](https://www.adafruit.com/product/292) with MCP23008 GPIO expander

_NOTE: This library is not made by Adafruit, and is not supported by them. The use of the Adafruit name
is for compatibility identification purposes only._

## Overview
This crate provides a driver for the Adafruit I2C LCD backpack with MCP23008 GPIO expander. It is designed to be used with the 
[embedded-hal](https://docs.rs/embedded-hal/latest/embedded_hal/index.html) traits for embeded systems. It supports standard
HD44780 based LCD displays.

## Features
The feature `shared_i2c` can be enabled to allow the I2C bus to be shared with other devices. This is useful for devices like the
Raspberry Pi Pico that have a single I2C bus. When this feature is enabled, the I2C bus is wrapped in an `Rc<RefCell<_>>` and
passed to the LCD backpack. When this feature is not enabled, the I2C bus is passed directly to the LCD backpack and consumed during
initialization.

## Usage
To create a new LCD backpack, use the `new` method. This will return a new LCD backpack object. Pass it the type of LCD display you
are using, the I2C bus, and the delay object. Both the I2C Bus and Delay objects must implement the relevant embedded-hal traits.

```rust
// The embedded-hal traits are used to define the I2C bus and delay objects
use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    blocking::i2c::{Write, WriteRead},
};
use lcd_backpack::{LcdBackpack, LcdDisplayType};

// create the I2C bus per your platform
let i2c = ...;

// create the delay object per your platform
let delay = ...;

// create the LCD backpack
let mut lcd = LcdBackpack::new(LcdDisplayType::Lcd16x2, i2c, delay);

// initialize the LCD
if let Err(_e) = lcd.init() {
   panic!("Error initializing LCD");
}
```
This library supports the `core::fmt::Write` trait, allowing it to be used with the `write!` macro. For example:
```rust
use core::fmt::Write;

// write a string to the LCD
if let Err(_e) = write!(lcd, "Hello, world!") {
  panic!("Error writing to LCD");
}
```
The various methods for controlling the LCD are also available. Each returns a `Result` that wraps the LCD backpack object. This
allows you to chain the methods together. For example:

```rust
// clear the display and home the cursor before writing a string
if let Err(_e) = write!(lcd.clear()?.home()?, "Hello, world!") {
 panic!("Error writing to LCD");
}
```
### Shared I2C Bus
If you are using a platform with a single I2C bus, you can enable the `shared_i2c` feature to allow the I2C bus to be shared with
other devices and sensors. That is, the I2C bus is wrapped in an `Rc<RefCell<_>>` and passed to the LCD backpack, meaning the
I2C bus object is not moved into the LCD backpack during initialization. Doing this requires that your project have an allocator
defined, such as [`embedded_alloc`](https://github.com/rust-embedded/embedded-alloc), allowing the use of the alloc::rc::Rc and 
core::cell::RefCell types.

When using the `shared_i2c` feature, the initial setup now looks like this (ignoring the creation of the allocator):

```rust
// The embedded-hal traits are used to define the I2C bus and delay objects
use embedded_hal::{
   blocking::delay::{DelayMs, DelayUs},
   blocking::i2c::{Write, WriteRead},
};

// The alloc crate is used to define the Rc and RefCell types
use alloc::rc::Rc;
use core::cell::RefCell;

use lcd_backpack::{LcdBackpack, LcdDisplayType};

// create the I2C bus per your platform
let i2c = ...;
let i2c = Rc::new(RefCell::new(i2c));

// create the delay object per your platform
let delay = ...;
let delay = Rc::new(RefCell::new(delay));

// create the LCD backpack
let mut lcd = LcdBackpack::new(LcdDisplayType::Lcd16x2, &i2c, &delay);

// initialize the LCD
if let Err(_e) = lcd.init() {
  panic!("Error initializing LCD");
}
```

<!-- cargo-sync-readme end -->

## License
Licensed under the [MIT](LICENSE-MIT) license.
