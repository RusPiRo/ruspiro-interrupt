# Changelog

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
