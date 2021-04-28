# Interrupt RusPiRo crate

This crates provides functions and macros (custom attributes) to conviniently define and implement interrupt handler for
the Raspberry Pi 3 in a bare metal environment.

![CI](https://github.com/RusPiRo/ruspiro-interrupt/workflows/CI/badge.svg?branch=development)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-interrupt.svg)](https://crates.io/crates/ruspiro-interrupt)
[![Documentation](https://docs.rs/ruspiro-interrupt/badge.svg)](https://docs.rs/ruspiro-interrupt)
[![License](https://img.shields.io/crates/l/ruspiro-interrupt.svg)](https://github.com/RusPiRo/ruspiro-interrupt#license)

## Dependencies

This crate, when used to build a final binary, assumes that there is a low level exeption handler in place that will call into a function called `__isr_default`. When using the [``ruspiro-boot`` crate](https://crates.io/crates/ruspiro-boot) this function is *weakly* exported from their to allow floawless compiling and linking. However, this empty export will be overruled by the linker with the function of the same name exported from this crate.

## Usage

To use the crate just add the following dependency to your ``Cargo.toml`` file:

```toml
[dependencies]
ruspiro-interrupt = "||VERSION||"
```

Once done the access to the features/attribute of the interrupt crate is available in your rust files like so:

```rust
extern crate ruspiro_interrupt; // needed for proper linking of weak defined functions
use ruspiro_interrupt::*;

#[IrqHandler(<irq-type-name>)]
unsafe fn my_handler(tx: Option<IsrSender<Box<dyn Any>>>) {
  /* implementation omitted */
}
```

In rare cases the interrupt line is shared for different sources, in this case the attribute need to specify the source:

```rust
#[IrqHandler(<irq-type-name>, <source>)]
unsafe fn my_handler_for_source(tx: Option<IsrSender<Box<dyn Any>>>) {
  /* implementation omitted */
}
```

The currently only implemented shared source interrupt line is the ``AUX`` interrupt. There the source could be one of:
``Uart1``, ``Spi1`` or ``Spi2``.

## Features

Feature   | Description
----------|-------------
**async** | Enables the `async` version of the interrupt handling implementation.

## License

Licensed under Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0) or MIT ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)) at your choice.
