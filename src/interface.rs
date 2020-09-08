/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Internal interrupt interface implementation
//!
use ruspiro_register::define_mmio_register;

#[cfg(feature = "ruspiro_pi3")]
const PERIPHERAL_BASE: u32 = 0x3F00_0000;

#[cfg(feature = "ruspiro_pi3")]
const ARM_CORE_BASE: u32 = 0x4000_0000;

const ARM_IRQ_BASE: u32 = PERIPHERAL_BASE + 0x0000_B000;

pub(crate) fn initialize() {
    // disable all interrupts in all 3 banks by default
    IRQ_DISABLE_1::Register.set(0xFFFF_FFFF);
    IRQ_DISABLE_2::Register.set(0xFFFF_FFFF);
    IRQ_DISABLE_B::Register.set(0xFFFF_FFFF);

    #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
    unsafe {
        llvm_asm!("dmb sy")
    };

    // set the routing of GPU interrupts to core 0
    GPU_INT_ROUTING::Register.set(0);

    // setup IPI (inter-processor-interrupts)
    // raising IRQ only if something is written to mailbox 3 for any of the cores
    CORE_MB_INT_CONTROL0::Register.set(1 << 3);
    CORE_MB_INT_CONTROL1::Register.set(1 << 3);
    CORE_MB_INT_CONTROL2::Register.set(1 << 3);
    CORE_MB_INT_CONTROL3::Register.set(1 << 3);
}

pub(crate) fn activate(bank: u32, irq_num: u32) {
    let enable_bit = 1 << (irq_num & 0x1F);
    match bank {
        0 => IRQ_ENABLE_1::Register.set(enable_bit),
        1 => IRQ_ENABLE_2::Register.set(enable_bit),
        2 => IRQ_ENABLE_B::Register.set(enable_bit),
        _ => (),
    }
}

pub(crate) fn deactivate(bank: u32, irq_num: u32) {
    let disable_bit = 1 << (irq_num & 0x1F);
    match bank {
        0 => IRQ_DISABLE_1::Register.set(disable_bit),
        1 => IRQ_DISABLE_2::Register.set(disable_bit),
        2 => IRQ_DISABLE_B::Register.set(disable_bit),
        _ => (),
    }
}

pub(crate) fn get_pending_irqs() -> [u32; 3] {
    let pendings: [u32; 3] = [
        IRQ_PENDING_1::Register.get(),
        IRQ_PENDING_2::Register.get(),
        IRQ_PENDING_B::Register.get(),
    ];

    pendings
}

define_mmio_register! [
    GPU_INT_ROUTING<ReadWrite<u32>@(ARM_CORE_BASE + 0x00C)>,

    CORE_MB_INT_CONTROL0<ReadWrite<u32>@(ARM_CORE_BASE + 0x050)>,
    CORE_MB_INT_CONTROL1<ReadWrite<u32>@(ARM_CORE_BASE + 0x054)>,
    CORE_MB_INT_CONTROL2<ReadWrite<u32>@(ARM_CORE_BASE + 0x058)>,
    CORE_MB_INT_CONTROL3<ReadWrite<u32>@(ARM_CORE_BASE + 0x05C)>,

    CORE_IRQ_PENDING0<ReadWrite<u32>@(ARM_CORE_BASE + 0x060)>,
    CORE_IRQ_PENDING1<ReadWrite<u32>@(ARM_CORE_BASE + 0x064)>,
    CORE_IRQ_PENDING2<ReadWrite<u32>@(ARM_CORE_BASE + 0x068)>,
    CORE_IRQ_PENDING3<ReadWrite<u32>@(ARM_CORE_BASE + 0x06C)>,

    IRQ_PENDING_B<ReadWrite<u32>@(ARM_IRQ_BASE + 0x200)>,
    IRQ_PENDING_1<ReadWrite<u32>@(ARM_IRQ_BASE + 0x204)>,
    IRQ_PENDING_2<ReadWrite<u32>@(ARM_IRQ_BASE + 0x208)>,
    
    FIQ_CONTROL<ReadWrite<u32>@(ARM_IRQ_BASE + 0x20C)>,

    IRQ_ENABLE_1<ReadWrite<u32>@(ARM_IRQ_BASE + 0x210)>,
    IRQ_ENABLE_2<ReadWrite<u32>@(ARM_IRQ_BASE + 0x214)>,
    IRQ_ENABLE_B<ReadWrite<u32>@(ARM_IRQ_BASE + 0x218)>,

    IRQ_DISABLE_1<ReadWrite<u32>@(ARM_IRQ_BASE + 0x21C)>,
    IRQ_DISABLE_2<ReadWrite<u32>@(ARM_IRQ_BASE + 0x220)>,
    IRQ_DISABLE_B<ReadWrite<u32>@(ARM_IRQ_BASE + 0x224)>
];
