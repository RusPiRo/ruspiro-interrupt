/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-interrupt/||VERSION||")]
#![no_std]
#![feature(asm)]
#![feature(linkage)]

//! # Interrupt handler for Raspberry Pi
//!
//! This crates provides functions and macros (custom attribute) to conviniently implement interrupt handler for
//! Raspberry Pi 3. The possible interrupts a handler can be implemented for are available as enum [irqtypes::Interrupt]
//!
//! # Usage
//!
//! ```no_run
//! extern crate ruspiro_interrupt; // <- this kind of usage is VERY IMPORTANT to ensure linking works as expected!
//! use ruspiro_interrupt::{self as irq, IrqHandler, IsrSender, isr_channel};
//!
//! #[IrqHandler(ArmTimer)]
//! fn timer_handler(channel: Option<IsrSender<Box<dyn Any>>>) {
//!     // IMPORTANT: acknowledge the irq !
//!
//!     // implement stuff that shall be executed if the interrupt is raised...
//!     // be careful when this code uses spinlocks as this might lead to dead-locks if the
//!     // executing code interrupted currently holds a lock the code inside this handler tries to aquire as well
//! }
//! ```
//!
//! In some cases the interrupt type/line is shared between different sources. In those cases a handler need to be
//! implemented for the specific interrupt source. The source is given in the custom attribute like this:
//!
//! ```no_run
//! extern crate ruspiro_interrupt; // <- this kind of usage is VERY IMPORTANT to ensure linking works as expected!
//! use ruspiro_interrupt::*;
//!
//! #[IrqHandler(Aux, Uart1)]
//! fn aux_uart1_handler(channel: Option<IsrSender<Box<dyn Any>>>) {
//!     // implement Uart1 interrupt handler here
//! }
//! ```
//!
//! With the actual interrupt handling routines in place the corresponding interrupts need to be configured and
//! activated like the following.
//!
//! ```no_run
//! fn main() {
//!     // as we have an interrupt handler defined we need to enable interrupt handling globally as well
//!     // as the specific interrupt we have a handler implemented for
//!     irq::initialize();
//!     // activate an irq that uses a channel to allow data to flow from the interrupt handler to the "normal"
//!     // processing
//!     let (timer_tx, mut timer_rx) = isr_channel::<()>();
//!     irq::activate(Interrupt::ArmTimer, timer_tx);
//!     // activate an irq that does not use a channel as all processing is done inside it's handler
//!     irq::activate(Interrupt::Aux, None);
//!
//!     enable_interrupts();
//!
//!     // wait for the interrupt to send some data along (blocking current execution)
//!     let _ = timer_rx.recv();
//!
//!     // when the crate is used with the feature `async` beeing set, waiting for
//!     // for the data send by the interrupt would look like this:
//!     while let Some(_) = timer_rx.next().await {
//!       // do stuff ...
//!     }
//! }
//! ```
//!
//! ## Limitations for shared interrupt lines
//!
//! However, only a limited ammount of shared interrupt lines implementation is available with the current version -
//! which is only the **Aux** interrupt at the moment.
//!
//!

#[cfg(not(any(feature = "pi3", feature = "pi4_low", feature = "pi4_high")))]
compile_error!("Either feature \"pi3\", \"pi4_low\" or \"pi4_high\" must be enabled for this crate");

extern crate alloc;
extern crate paste;

mod auxhandler;
mod bitset;
mod interface;
mod irqtypes;

use alloc::boxed::Box;
use auxhandler::set_aux_isrsender;
pub use auxhandler::AuxDevice;
use core::{any::Any, cell::RefCell};
pub use irqtypes::Interrupt;
pub use ruspiro_interrupt_macros::IrqHandler;

#[cfg(feature = "async")]
pub use ruspiro_channel::mpmc::async_channel as isr_channel;
#[cfg(not(feature = "async"))]
pub use ruspiro_channel::mpmc::channel as isr_channel;
#[cfg(feature = "async")]
pub use ruspiro_channel::mpmc::AsyncReceiver as IsrReceiver;
#[cfg(feature = "async")]
pub use ruspiro_channel::mpmc::AsyncSender as IsrSender;
#[cfg(not(feature = "async"))]
pub use ruspiro_channel::mpmc::Receiver as IsrReceiver;
#[cfg(not(feature = "async"))]
pub use ruspiro_channel::mpmc::Sender as IsrSender;

type IsrChannel = Option<IsrSender<Box<dyn Any>>>;

/// One time interrupt manager initialization. This performs the initial configuration and deactivates all IRQs
pub fn initialize() {
  interface::initialize();
}

/// globally enabling interrupts (IRQ/FIQ) to be triggered
pub fn enable_interrupts() {
  interface::enable_irq();
  interface::enable_fiq();
}

/// globally disabling interrupts (IRQ/FIQ) from beeing triggered
pub fn disable_interrupts() {
  interface::disable_irq();
  interface::disable_fiq();
}

/// Activate a specific interrupt to be raised and handled (if a handler is implemented).
/// If there is no handler implemented for this interrupt it may lead to an endless interrupt
/// loop as the interrupt never gets acknowledged by the handler.
/// There is unfortunately no generic way of acknowledgement implementation possible as the acknowledge
/// register and process differs for the individual interrupts and thus need to be implemented in the specific
/// handler.
///
/// You might want to pass a sender of an interrupt service routine channel. This channel can be used within the
/// interrupt handler implementation to pass data from the ISR to the normal processing. An example for this could be
/// an interrupt handler triggered by incomming data on the UART peripheral. The interrupt handler could read the
/// incomming data and push it into the channel for further processing that should take place outside of the interrupt
/// handler because this one should run as fast as possible.
/// To register an interrupt handler for a shared interrupt line the specialized respective function should be used.
///
/// # Panics
/// The function panics if it is called for a known shared interrupt line
///
pub fn activate(irq: Interrupt, channel: IsrChannel) {
  // Aux interrupts share one interrupt line - thus special handling for setting the IsrSender
  // Aux interrupt activation is done in a separate function
  if irq == Interrupt::Aux {
    panic!("AUX interrupts require activation with 'activate_aux'");
  }

  let irq_bank = (irq as u32) >> 5;
  let irq_num = (irq as u32) & 0x1F;

  ISR_LIST.0.get(irq_bank as usize).map(|bank| {
    bank
      .get(irq_num as usize)
      .map(|(_, irq_channel)| *irq_channel.borrow_mut() = channel);
  });

  interface::activate(irq);
  #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
  unsafe {
    asm!("dmb sy")
  };
}

/// Activate the AUX interrupt line. This line is shared between three aux devices. The miniUART, SPI1 and SPI2.
/// The interrupts for those devices can't be enabled individually. However, we allow to register different IsrSender
/// for the individual device as the interrupt provisioning is based on the AUXIRQ status register that indicates the
/// correct device having raised the interrupt. The only way to suppress interrups for an individual device would
/// require disabling of the device with the AUXENB register.
///
/// You might want to pass a sender of an interrupt service routine channel. This channel can be used within the
/// interrupt handler implementation to pass data from the ISR to the normal processing. An example for this could be
/// an interrupt handler triggered by incomming data on the UART peripheral. The interrupt handler could read the
/// incomming data and push it into the channel for further processing that should take place outside of the interrupt
/// handler because this one should run as fast as possible.
/// To register an interrupt handler for a shared interrupt line the specialized respective function should be used.
pub fn activate_aux(aux: AuxDevice, channel: IsrChannel) {
  // Aux interrupts share one interrupt line - thus special handling for setting the IsrSender
  set_aux_isrsender(aux, channel);

  interface::activate(Interrupt::Aux);
  #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
  unsafe {
    asm!("dmb sy")
  };
}

/// deactivate a specific interrupt from beeing raised. This ensures the handler will also not getting called any
/// longer
pub fn deactivate(irq: Interrupt) {
  interface::deactivate(irq);

  let irq_bank = (irq as u32) >> 5;
  let irq_num = (irq as u32) & 0x1F;

  ISR_LIST.0.get(irq_bank as usize).map(|bank| {
    bank
      .get(irq_num as usize)
      .map(|(_, irq_channel)| irq_channel.borrow_mut().take());
  });

  #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
  unsafe {
    asm!("dmb sy")
  };
}

/********************************************************************************************
 * Functions that need to be exported, this seem not to work if they are part of of a child
 * module, so define them here
 ********************************************************************************************/
#[no_mangle]
unsafe extern "C" fn __isr_default() {
  // now retrieve the pending interrupts (already filtered by the active one)
  let pendings = interface::get_pending_irqs();
  // now dispatch the interrupts to their respective handler
  for (&pending_bank, handler_bank) in pendings.iter().zip(ISR_LIST.0.iter()) {
    for irq in bitset::BitSet32(pending_bank).iter() {
      handler_bank.get(irq as usize).map(|(handler, channel)| {
        handler(channel.borrow().clone());
      });
    }
  }
}

macro_rules! default_handler_impl {
    ($($name:ident),*) => {$(
        paste::item!{
            #[allow(non_snake_case, improper_ctypes_definitions)]
            #[linkage="weak"]
            #[no_mangle]
            extern "C" fn [<__irq_handler__ $name>](_tx: IsrChannel){}
        }
    )*};
}

default_handler_impl![
  SystemTimer1,
  SystemTimer3,
  Isp,
  Usb,
  CoreSync0,
  CoreSync1,
  CoreSync2,
  CoreSync3,
  Aux_Uart1,
  Aux_Spi1,
  Aux_Spi2,
  Arm,
  GpuDma,
  GpioBank0,
  GpioBank1,
  GpioBank2,
  GpioBank3,
  I2c,
  Spi,
  I2sPcm,
  Sdio,
  Pl011,
  ArmTimer,
  ArmMailbox,
  ArmDoorbell0,
  ArmDoorbell1,
  ArmGpu0Halted,
  ArmGpu1Halted,
  ArmIllegalType1,
  ArmIllegalType0,
  ArmPending1,
  ArmPending2,
  CntPsIrq,
  CntPnsIrq,
  CntHpIrq,
  CntVIrq,
  Core0Mailbox3,
  Core1Mailbox3,
  Core2Mailbox3,
  Core3Mailbox3,
  CoreGPU,
  LocalTimer
];

#[allow(non_snake_case, improper_ctypes_definitions)]
#[no_mangle]
extern "C" fn __irq_handler_Default(_channel: IsrChannel) {}

struct IsrList([[(extern "C" fn(IsrChannel), RefCell<IsrChannel>); 32]; 4]);
unsafe impl Sync for IsrList {}

/// The list of interrupt service routines
static ISR_LIST: IsrList = IsrList([
  [
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler__SystemTimer1, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler__SystemTimer3, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler__Isp, RefCell::new(None)),
    (__irq_handler__Usb, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)), //10
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler__CoreSync0, RefCell::new(None)),
    (__irq_handler__CoreSync1, RefCell::new(None)),
    (__irq_handler__CoreSync2, RefCell::new(None)),
    (__irq_handler__CoreSync3, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)), //20
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (auxhandler::aux_handler, RefCell::new(None)),
    (__irq_handler__Arm, RefCell::new(None)), //30
    (__irq_handler__GpuDma, RefCell::new(None)),
  ],
  [
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)), // 40
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler__GpioBank0, RefCell::new(None)),
    (__irq_handler__GpioBank1, RefCell::new(None)), // 50
    (__irq_handler__GpioBank2, RefCell::new(None)),
    (__irq_handler__GpioBank3, RefCell::new(None)),
    (__irq_handler__I2c, RefCell::new(None)),
    (__irq_handler__Spi, RefCell::new(None)),
    (__irq_handler__I2sPcm, RefCell::new(None)),
    (__irq_handler__Sdio, RefCell::new(None)),
    (__irq_handler__Pl011, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)), // 60
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
  ],
  [
    (__irq_handler__ArmTimer, RefCell::new(None)),
    (__irq_handler__ArmMailbox, RefCell::new(None)),
    (__irq_handler__ArmDoorbell0, RefCell::new(None)),
    (__irq_handler__ArmDoorbell1, RefCell::new(None)),
    (__irq_handler__ArmGpu0Halted, RefCell::new(None)),
    (__irq_handler__ArmGpu1Halted, RefCell::new(None)),
    (__irq_handler__ArmIllegalType1, RefCell::new(None)), // 70
    (__irq_handler__ArmIllegalType0, RefCell::new(None)),
    (__irq_handler__ArmPending1, RefCell::new(None)),
    (__irq_handler__ArmPending2, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)), // 80
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)), // 90
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
  ],
  [
    (__irq_handler__CntPsIrq, RefCell::new(None)),
    (__irq_handler__CntPnsIrq, RefCell::new(None)),
    (__irq_handler__CntHpIrq, RefCell::new(None)),
    (__irq_handler__CntVIrq, RefCell::new(None)),
    (__irq_handler__Core0Mailbox3, RefCell::new(None)), // 100
    (__irq_handler__Core1Mailbox3, RefCell::new(None)),
    (__irq_handler__Core2Mailbox3, RefCell::new(None)),
    (__irq_handler__Core3Mailbox3, RefCell::new(None)),
    (__irq_handler__CoreGPU, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler__LocalTimer, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)), // 110
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)), // 120
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
    (__irq_handler_Default, RefCell::new(None)),
  ],
]);
