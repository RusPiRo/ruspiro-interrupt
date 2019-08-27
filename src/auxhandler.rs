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

use ruspiro_register::define_registers;

#[cfg(feature="ruspiro_pi3")]
const PERIPHERAL_BASE: u32 = 0x3F00_0000;

pub(crate) fn aux_handler() {
    // special Aux handling, as one IRQ line shares interrupts between Uart1, SPI1 and SPI2
    if AUX_IRQ::Register.read(AUX_IRQ::UART1) == 1 {
        crate::__irq_handler__Aux_Uart1();
    }

    if AUX_IRQ::Register.read(AUX_IRQ::SPI1) == 1 {
        crate::__irq_handler__Aux_Spi1();
    }

    if AUX_IRQ::Register.read(AUX_IRQ::SPI2) == 1 {
        crate::__irq_handler__Aux_Spi2();
    }
}

define_registers! [
    AUX_IRQ: ReadWrite<u32> @ PERIPHERAL_BASE + 0x0021_5000 => [
        SPI2    OFFSET(2),
        SPI1    OFFSET(1),
        UART1   OFFSET(0)
    ]
];