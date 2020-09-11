# Interrupt RusPiRo crate

This crates provides functions and macros (custom attributes) to conviniently define and implement interrupt handler for
the Raspberry Pi 3 in a bare metal environment.

[![Travis-CI Status](https://api.travis-ci.org/RusPiRo/ruspiro-interrupt.svg?branch=master)](https://travis-ci.org/RusPiRo/ruspiro-interrupt)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-interrupt.svg)](https://crates.io/crates/ruspiro-interrupt)
[![Documentation](https://docs.rs/ruspiro-interrupt/badge.svg)](https://docs.rs/ruspiro-interrupt)
[![License](https://img.shields.io/crates/l/ruspiro-interrupt.svg)](https://github.com/RusPiRo/ruspiro-interrupt#license)

## Dependencies
This crate, when used to build a final binarry assumes that certain linker symbols does exist and are defined. These are
usually provided when using the [``ruspiro-boot`` crate](https://crates.io/crates/ruspiro-boot) which comes with a linker
script providing all the necessary linker symbols and entrypoints calling into this crate once it is used.

## Usage
To use the crate just add the following dependency to your ``Cargo.toml`` file:

```toml
[dependencies]
ruspiro-interrupt = "0.3"
```

Once done the access to the features/attribute of the interrupt crate is available in your rust files like so:

```rust
extern crate ruspiro_interrupt; // needed for proper linking of weak defined functions
use ruspiro-interrupt::*;

#[IrqHandler(<irq-type-name>)]
unsafe fn my_handler() {

}
```

In rare cases the interrupt line is shared for different sources, in this case the attribute need to specify the source:

```rust
#[IrqHandler(<irq-type-name>, <source>)]
unsafe fn my_handler_for_source() {

}
```

The currently only implemented shared source interrupt line is the ``AUX`` interrupt. There the source could be one of:
``Uart1``, ``Spi1`` or ``Spi2``.

## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)