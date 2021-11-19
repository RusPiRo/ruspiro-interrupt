# Changelog

## :mouse: v0.5.0

- ### :wrench: Maintenance

  Clean out feature names:
  - `pi3`: is now the feature to use the Raspberry Pi3 peripheral addresses
  - `pi4_low`: use the Raspberry Pi4 Low-Peri mode peripheral addresses
  - `pi4_high`: use the Raspberry Pi4 High-Peri mode peripheral addresses

- ### :bulb: Features

  Adding support for Raspberry Pi4 interrupt handling using the legacy interrupt controller.
  
## :dog: v0.4.2

- ### :wrench: Maintenance

  Some minor cleanup has been done. The major addition is to provide a minimal example of how the crate is intended to be used to implement an interrupt handler based on the ARM system timer.

## :cat: v0.4.1

- ### :bulb: Features
  
  Rework the interrupt crate to support sync and async interrupt processing. Each interrupt handler implementation will recieve a `Sender` of a specific channel that may have been passed to the interrupt handling while activating a specific interrupt. The signature of a interrupt handling function will thus now look like:
  
  ```rust
  #[IrqHandler(Foo)]
  fn foo_handler(tx: Option<IsrSender<Box<dyn Any>>>) { }
  ```
