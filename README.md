# Interrupt RusPiRo crate

## Usage
To use the crate just add the following dependency to your ``Cargo.toml`` file:
```
[dependencies]
ruspiro-interrupt = "0.0.1"
```

Once done the access to the features/attribute of the interrupt crate is available in your rust files like so:
```
use ruspiro-interrupt::*;

#[IrqHandler(<irq-type-name>)]
unsafe fn my_handler() {

}
```

## License
Licensed under Apache License, Version 2.0, ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)