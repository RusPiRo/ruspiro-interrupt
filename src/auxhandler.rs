/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/
//! # Aux interrupt line handler
//!
//! The Aux interrupt line is shared between Uart1, Spi1 and Spi2. This handler branches to the specific handler
//! implementation based on the interrupt source.
//!

use alloc::boxed::Box;
use core::{any::Any, cell::RefCell};
use ruspiro_mmio_register::define_mmio_register;

#[cfg(feature = "async")]
use ruspiro_channel::mpmc::AsyncSender as IsrSender;
#[cfg(not(feature = "async"))]
use ruspiro_channel::mpmc::Sender as IsrSender;

#[cfg(feature = "ruspiro_pi3")]
const PERIPHERAL_BASE: usize = 0x3F00_0000;

pub enum AuxDevice {
  Uart1,
  Spi1,
  Spi2,
}

pub(crate) fn set_aux_isrsender(aux: AuxDevice, tx: IsrSender<Box<dyn Any>>) {
  match aux {
    AuxDevice::Uart1 => AUXISRSENDER.uart1.borrow_mut().replace(tx),
    AuxDevice::Spi1 => AUXISRSENDER.spi1.borrow_mut().replace(tx),
    AuxDevice::Spi2 => AUXISRSENDER.spi2.borrow_mut().replace(tx),
  };
}

pub(crate) extern "C" fn aux_handler(_tx: IsrSender<Box<dyn Any>>) {
  // special Aux handling, as one IRQ line shares interrupts between Uart1, SPI1 and SPI2
  if AUX_IRQ::Register.read(AUX_IRQ::UART1) == 1 {
    AUXISRSENDER
      .uart1
      .borrow()
      .as_ref()
      .map(|tx| crate::__irq_handler__Aux_Uart1(tx.clone()));
  }

  if AUX_IRQ::Register.read(AUX_IRQ::SPI1) == 1 {
    AUXISRSENDER
      .spi1
      .borrow()
      .as_ref()
      .map(|tx| crate::__irq_handler__Aux_Spi1(tx.clone()));
  }

  if AUX_IRQ::Register.read(AUX_IRQ::SPI2) == 1 {
    AUXISRSENDER
      .spi2
      .borrow()
      .as_ref()
      .map(|tx| crate::__irq_handler__Aux_Spi2(tx.clone()));
  }
}

struct AuxIsrSender {
  uart1: RefCell<Option<IsrSender<Box<dyn Any>>>>,
  spi1: RefCell<Option<IsrSender<Box<dyn Any>>>>,
  spi2: RefCell<Option<IsrSender<Box<dyn Any>>>>,
}

unsafe impl Sync for AuxIsrSender {}

static AUXISRSENDER: AuxIsrSender = AuxIsrSender {
  uart1: RefCell::new(None),
  spi1: RefCell::new(None),
  spi2: RefCell::new(None),
};

define_mmio_register! [
    AUX_IRQ<ReadWrite<u32>@(PERIPHERAL_BASE + 0x0021_5000)> {
        SPI2    OFFSET(2),
        SPI1    OFFSET(1),
        UART1   OFFSET(0)
    }
];
