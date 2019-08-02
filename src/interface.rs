/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Internal interrupt interface implementation
//! 

use ruspiro_register::define_registers;

//#[cfg(feature="ruspiro-pi2")]
//const PERIPHERAL_BASE: u32 = 0x2000_0000;

#[cfg(feature="ruspiro-pi3")]
const PERIPHERAL_BASE: u32 = 0x3F00_0000;

#[cfg(feature="ruspiro-pi3")]
const ARM_CORE_BASE: u32 = 0x4000_0000;

const ARM_IRQ_BASE: u32 = PERIPHERAL_BASE + 0x0000_B200;

pub(crate) fn initialize() {
    // disable all interrupts in all 3 banks by default
    IRQ_DISABLE_1::Register.set(0xFFFF_FFFF);
    IRQ_DISABLE_2::Register.set(0xFFFF_FFFF);
    IRQ_DISABLE_B::Register.set(0xFFFF_FFFF);
    unsafe{ asm!("dmb") };

    // set the routing of GPU interrupts to core 0
    GPU_INT_ROUTING::Register.set(0);

    // setup IPI (inter-processor-interrupts)
    // raising IRQ only if something is written to mailbox 3 for any of the cores
    CORE_MB_INT_CONTROL0::Register.set(1 << 3);
    CORE_MB_INT_CONTROL1::Register.set(1 << 3);
    CORE_MB_INT_CONTROL2::Register.set(1 << 3);
    CORE_MB_INT_CONTROL3::Register.set(1 << 3);
}

pub(crate) fn enable_i() {
    unsafe { asm!("cpsie i") };
}

pub(crate) fn enable_f() {
    unsafe { asm!("cpsie f") };
}

pub(crate) fn disable_i() {
    unsafe { asm!("cpsid i") };
}

pub(crate) fn disable_f() {
    unsafe { asm!("cpsid f") };
}

define_registers! [
    GPU_INT_ROUTING: ReadWrite<u32> @ ARM_CORE_BASE + 0x0C => [],

    CORE_MB_INT_CONTROL0: ReadWrite<u32> @ ARM_CORE_BASE + 0x50 => [],
    CORE_MB_INT_CONTROL1: ReadWrite<u32> @ ARM_CORE_BASE + 0x54 => [],
    CORE_MB_INT_CONTROL2: ReadWrite<u32> @ ARM_CORE_BASE + 0x58 => [],
    CORE_MB_INT_CONTROL3: ReadWrite<u32> @ ARM_CORE_BASE + 0x5C => [],

    CORE_IRQ_PENDING0: ReadWrite<u32> @ ARM_CORE_BASE + 0x60 => [],
    CORE_IRQ_PENDING1: ReadWrite<u32> @ ARM_CORE_BASE + 0x64 => [],
    CORE_IRQ_PENDING2: ReadWrite<u32> @ ARM_CORE_BASE + 0x68 => [],
    CORE_IRQ_PENDING3: ReadWrite<u32> @ ARM_CORE_BASE + 0x6C => [],

    IRQ_PENDING_B: ReadWrite<u32> @ ARM_IRQ_BASE + 0x00 => [],
    IRQ_PENDING_1: ReadWrite<u32> @ ARM_IRQ_BASE + 0x04 => [],
    IRQ_PENDING_2: ReadWrite<u32> @ ARM_IRQ_BASE + 0x08 => [],
    
    FIQ_CONTROL: ReadWrite<u32> @ ARM_IRQ_BASE + 0x0C => [],

    IRQ_ENABLE_1: ReadWrite<u32> @ ARM_IRQ_BASE + 0x10 => [],
    IRQ_ENABLE_2: ReadWrite<u32> @ ARM_IRQ_BASE + 0x14 => [],
    IRQ_ENABLE_B: ReadWrite<u32> @ ARM_IRQ_BASE + 0x18 => [],

    IRQ_DISABLE_1: ReadWrite<u32> @ ARM_IRQ_BASE + 0x1C => [],
    IRQ_DISABLE_2: ReadWrite<u32> @ ARM_IRQ_BASE + 0x20 => [],
    IRQ_DISABLE_B: ReadWrite<u32> @ ARM_IRQ_BASE + 0x24 => []
];