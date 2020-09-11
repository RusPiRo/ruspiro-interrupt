# RusPiRo core interrupt crate

Core interrupt functions to globally enable/disable interrupts to be triggered on Raspberry Pi. Splitting from the
[``ruspiro-interrupt`` crate](https://crates.io/crates/ruspiro-interrupt) is necessary to prevent circular dependencies.

[![Travis-CI Status](https://api.travis-ci.org/RusPiRo/ruspiro-interrupt.svg?branch=master)](https://travis-ci.org/RusPiRo/ruspiro-interrupt)
[![Latest Version](https://img.shields.io/crates/v/ruspiro-interrupt-core.svg)](https://crates.io/crates/ruspiro-interrupt-core)
[![Documentation](https://docs.rs/ruspiro-interrupt-core/badge.svg)](https://docs.rs/ruspiro-interrupt-core)
[![License](https://img.shields.io/crates/l/ruspiro-interrupt-core.svg)](https://github.com/RusPiRo/ruspiro-interrupt-core#license)

## Usage

To use the crate just add the following dependency to your ``Cargo.toml`` file:

```toml
[dependencies]
ruspiro-interrupt-core = "0.3"
```

Once done the access to the functions to enable/disable interrupts is available in your rust files like so:

```rust
use ruspiro_interrupt_core::*;

fn demo() {
    enable_interrupts();
    disable_interrupts();
    re_enable_interrupts();
}

```

## License

Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)