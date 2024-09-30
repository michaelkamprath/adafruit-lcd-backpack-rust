# adafruit-lcd-backpack

## THIS LIBRARY IS NO LONGER MAINTAINED
This library has not been updated to be compatible with `embedded-hal` version 1.0.0 and later. It is recommended to use the [`i2c-character-display`](https://crates.io/crates/i2c-character-display) driver instead ([repository](https://github.com/michaelkamprath/i2c-character-display)), which is compatible with `embedded-hal` version 1.0.0 and later and supports the Adafruit I2C LCD backpack plus other I2C character displays.

---
<!-- cargo-sync-readme start -->

Rust driver for the [Adafruit I2C LCD backpack](https://www.adafruit.com/product/292) with MCP23008 GPIO expander

_NOTE: This library is not made by Adafruit, and is not supported by them. The use of the Adafruit name
is for compatibility identification purposes only._

## Overview
This crate provides a driver for the Adafruit I2C LCD backpack with MCP23008 GPIO expander. It is designed to be used with the
[embedded-hal](https://docs.rs/embedded-hal/latest/embedded_hal/index.html) traits for embeded systems. It supports standard
HD44780 based LCD displays.

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

<!-- cargo-sync-readme end -->

## License
Licensed under the [MIT](LICENSE-MIT) license.
