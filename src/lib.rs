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

use ruspiro_register::define_registers;

