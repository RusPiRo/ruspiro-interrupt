# Changelog

## :cat: v0.4.1

- ### :bulb: Features
  
  Rework the interrupt crate to support sync and async interrupt processing. Each interrupt handler implementation will recieve a `Sender` of a specific channel that may have been passed to the interrupt handling while activating a specific interrupt. The signature of a interrupt handling function will thus now look like:
  
  ```rust
  #[IrqHandler(Foo)]
  fn foo_handler(tx: Option<IsrSender<Box<dyn Any>>>) { }
  ```
