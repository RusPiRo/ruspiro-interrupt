/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/
#![doc(html_root_url = "https://docs.rs/ruspiro-interrupt/0.0.1")]
#![no_std]
#![feature(asm)]

//! # Raspberry Pi Interrupt handler
//! 

pub use ruspiro_interrupt_macros::*;
pub mod irqtypes;

use ruspiro_singleton::Singleton;
mod interface;

/// The singleton accessor to the interrupt manager
pub static INTERRUPTMGR: Singleton<InterruptManager> = Singleton::new(InterruptManager::new());

/// The interrupt manager representation
pub struct InterruptManager {
    enabled: [u32; 3], // store the current active irq's of the different banks (shadowing the register value)
}

impl InterruptManager {
    pub(crate) const fn new () -> Self {
        InterruptManager {
            enabled: [0; 3],
        }
    }

    /// One time interrupt manager initialization. This performs the initial configuration and deactivates all IRQs
    pub fn initialize(&mut self) {
        interface::initialize();
    }

    /// globally enable interrupts
    pub fn enable(&self) {
        interface::enable_i();
    }

    /// globally disable interrupts
    pub fn disable(&self) {
        interface::disable_i();
    }
}


// define the registers for