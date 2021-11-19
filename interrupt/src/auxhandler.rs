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

use super::IsrChannel;
use core::{cell::RefCell};
use ruspiro_mmio_register::define_mmio_register;

#[cfg(feature = "pi3")]
const PERIPHERAL_BASE: usize = 0x0_3F00_0000;
#[cfg(feature = "pi4_low")]
const PERIPHERAL_BASE: usize = 0x0_FE00_0000;
#[cfg(feature = "pi4_high")]
const PERIPHERAL_BASE: usize = 0x4_7E00_0000;

pub enum AuxDevice {
  Uart1,
  Spi1,
  Spi2,
}

pub(crate) fn set_aux_isrsender(aux: AuxDevice, channel: IsrChannel) {
  if let Some(channel) = channel {
    match aux {
      AuxDevice::Uart1 => AUXISRSENDER.uart1.borrow_mut().replace(channel),
      AuxDevice::Spi1 => AUXISRSENDER.spi1.borrow_mut().replace(channel),
      AuxDevice::Spi2 => AUXISRSENDER.spi2.borrow_mut().replace(channel),
    };
  }
}

#[allow(improper_ctypes_definitions)]
pub(crate) extern "C" fn aux_handler(_: IsrChannel) {
  // special Aux handling, as one IRQ line shares interrupts between Uart1, SPI1 and SPI2
  if AUX_IRQ::Register.read(AUX_IRQ::UART1) == 1 {
    let channel = AUXISRSENDER.uart1.borrow().clone();
    crate::__irq_handler__Aux_Uart1(channel);
  }

  if AUX_IRQ::Register.read(AUX_IRQ::SPI1) == 1 {
    let channel = AUXISRSENDER.spi1.borrow().clone();
    crate::__irq_handler__Aux_Spi1(channel);
  }

  if AUX_IRQ::Register.read(AUX_IRQ::SPI2) == 1 {
    let channel = AUXISRSENDER.spi2.borrow().clone();
    crate::__irq_handler__Aux_Spi2(channel);
  }
}

struct AuxIsrSender {
  uart1: RefCell<IsrChannel>,
  spi1: RefCell<IsrChannel>,
  spi2: RefCell<IsrChannel>,
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
