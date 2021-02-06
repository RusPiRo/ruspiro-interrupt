/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Internal interrupt interface implementation
//!
use ruspiro_mmio_register::define_mmio_register;

use crate::Interrupt;

#[cfg(feature = "ruspiro_pi3")]
const PERIPHERAL_BASE: usize = 0x3F00_0000;

#[cfg(feature = "ruspiro_pi3")]
const ARM_CORE_BASE: usize = 0x4000_0000;

const ARM_IRQ_BASE: usize = PERIPHERAL_BASE + 0x0000_B000;

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

/// globally enable ``IRQ`` interrupts to be triggered
pub(crate) fn enable_irq() {
  #[cfg(target_arch = "aarch64")]
  unsafe {
    llvm_asm!(
      "msr daifclr, #2
            isb"
    ) // as per ARM spec the ISB ensures triggering pending interrupts
  };
}

/// globally enable ``FIQ`` interrupts to be triggered
pub(crate) fn enable_fiq() {
  #[cfg(target_arch = "aarch64")]
  unsafe {
    llvm_asm!(
      "msr daifclr, #1
            isb"
    ) // as per ARM spec the ISB ensures triggering pending interrupts
  };
}

/// globally disable ``IRQ`` interrupts from beeing triggered.
pub fn disable_irq() {
  #[cfg(target_arch = "aarch64")]
  unsafe {
    llvm_asm!("msr daifset, #2")
  };
}

/// globally disable ``FIQ`` interrupts from beeing triggered.
pub fn disable_fiq() {
  #[cfg(target_arch = "aarch64")]
  unsafe {
    llvm_asm!("msr daifset, #1")
  };
}

pub(crate) fn activate(irq: Interrupt) {
  let bank = (irq as u32) >> 5;
  let enable_bit = 1 << ((irq as u32) & 0x1F);
  match bank {
    0 => IRQ_ENABLE_1::Register.set(enable_bit),
    1 => IRQ_ENABLE_2::Register.set(enable_bit),
    2 => IRQ_ENABLE_B::Register.set(enable_bit),
    3 => {
      // this bank is special as it covers the Core specific interrupts that are
      // configured for their specific device
      match irq {
        Interrupt::CntPsIrq => {
          CORE0_TIMER_IRQ::Register.modify_value(CORE0_TIMER_IRQ::CNTPSIRQ::ENABLED);
        }
        Interrupt::CntPnsIrq => {
          CORE0_TIMER_IRQ::Register.modify_value(CORE0_TIMER_IRQ::CNTPNSIRQ::ENABLED);
        }
        Interrupt::CntHpIrq => {
          CORE0_TIMER_IRQ::Register.modify_value(CORE0_TIMER_IRQ::CNTHPIRQ::ENABLED);
        }
        Interrupt::CntVIrq => {
          CORE0_TIMER_IRQ::Register.modify_value(CORE0_TIMER_IRQ::CNTVIRQ::ENABLED);
        }
        Interrupt::Core0Mailbox3 => {
          CORE0_MAILBOX_IRQ::Register.modify_value(CORE0_MAILBOX_IRQ::MB3_IRQ::ENABLED);
        }
        Interrupt::Core1Mailbox3 => {
          CORE1_MAILBOX_IRQ::Register.modify_value(CORE1_MAILBOX_IRQ::MB3_IRQ::ENABLED);
        }
        Interrupt::Core2Mailbox3 => {
          CORE2_MAILBOX_IRQ::Register.modify_value(CORE2_MAILBOX_IRQ::MB3_IRQ::ENABLED);
        }
        Interrupt::Core3Mailbox3 => {
          CORE3_MAILBOX_IRQ::Register.modify_value(CORE3_MAILBOX_IRQ::MB3_IRQ::ENABLED);
        }
        Interrupt::CoreGPU => (), // seems GPU interrupt cant be enabled/disabled as they are triggered from GPU
        //Interrupt::CorePMU => (),
        //Interrupt::CoreAxi => (),
        Interrupt::LocalTimer => {
          LOCAL_TIMER_CTRL::Register.modify_value(LOCAL_TIMER_CTRL::IRQ_ENABLE::ENABLED);
        }
        _ => (),
      };
    }
    _ => (),
  }
}

pub(crate) fn deactivate(irq: Interrupt) {
  let bank = (irq as u32) >> 5;
  let disable_bit = 1 << ((irq as u32) & 0x1F);
  match bank {
    0 => IRQ_DISABLE_1::Register.set(disable_bit),
    1 => IRQ_DISABLE_2::Register.set(disable_bit),
    2 => IRQ_DISABLE_B::Register.set(disable_bit),
    3 => {
      // this bank is special as it covers the Core specific interrupts that are
      // configured for their specific device
      match irq {
        Interrupt::CntPsIrq => {
          CORE0_TIMER_IRQ::Register.modify_value(CORE0_TIMER_IRQ::CNTPSIRQ::DISABLED);
        }
        Interrupt::CntPnsIrq => {
          CORE0_TIMER_IRQ::Register.modify_value(CORE0_TIMER_IRQ::CNTPNSIRQ::DISABLED);
        }
        Interrupt::CntHpIrq => {
          CORE0_TIMER_IRQ::Register.modify_value(CORE0_TIMER_IRQ::CNTHPIRQ::DISABLED);
        }
        Interrupt::CntVIrq => {
          CORE0_TIMER_IRQ::Register.modify_value(CORE0_TIMER_IRQ::CNTVIRQ::DISABLED);
        }
        Interrupt::Core0Mailbox3 => {
          CORE0_MAILBOX_IRQ::Register.modify_value(CORE0_MAILBOX_IRQ::MB3_IRQ::DISABLED);
        }
        Interrupt::Core1Mailbox3 => {
          CORE1_MAILBOX_IRQ::Register.modify_value(CORE1_MAILBOX_IRQ::MB3_IRQ::DISABLED);
        }
        Interrupt::Core2Mailbox3 => {
          CORE2_MAILBOX_IRQ::Register.modify_value(CORE2_MAILBOX_IRQ::MB3_IRQ::DISABLED);
        }
        Interrupt::Core3Mailbox3 => {
          CORE3_MAILBOX_IRQ::Register.modify_value(CORE3_MAILBOX_IRQ::MB3_IRQ::DISABLED);
        }
        Interrupt::CoreGPU => (), // seems GPU interrupt cant be enabled/disabled as they are triggered from GPU
        //Interrupt::CorePMU => (),
        //Interrupt::CoreAxi => (),
        Interrupt::LocalTimer => {
          LOCAL_TIMER_CTRL::Register.modify_value(LOCAL_TIMER_CTRL::IRQ_ENABLE::DISABLED);
        }
        _ => (),
      };
    }
    _ => (),
  }
}

pub fn get_pending_irqs() -> [u32; 4] {
  let pendings: [u32; 4] = [
    // grab the common pending register contents
    IRQ_PENDING_1::Register.get() & IRQ_ENABLE_1::Register.get(),
    IRQ_PENDING_2::Register.get() & IRQ_ENABLE_2::Register.get(),
    IRQ_PENDING_B::Register.get() & IRQ_ENABLE_B::Register.get(),
    // each core does have additional dedicated pending registers, which IRQ's
    // does not seem to appear in the other ones
    // as the most core specific interrupts can occur in one core only we merge them
    // into a single pending value, assuming we can ignore which core the interrupt was
    // routed to - with the exception of the mailbox interrupts. We allow only mailbox 3
    // interrupts on each core, as they share the same bit we need to shift this ot the correct
    // position
    (CORE0_IRQ_PENDING::Register.get()
      | CORE1_IRQ_PENDING::Register.get()
      | CORE2_IRQ_PENDING::Register.get()
      | CORE3_IRQ_PENDING::Register.get())
      & !(0b1111 << 4)
      | (CORE0_IRQ_PENDING::Register
        .read_value(CORE0_IRQ_PENDING::MB3_IRQ)
        .raw_value()
        >> 3)
      | (CORE1_IRQ_PENDING::Register
        .read_value(CORE1_IRQ_PENDING::MB3_IRQ)
        .raw_value()
        >> 2)
      | (CORE2_IRQ_PENDING::Register
        .read_value(CORE2_IRQ_PENDING::MB3_IRQ)
        .raw_value()
        >> 1)
      | (CORE3_IRQ_PENDING::Register
        .read_value(CORE3_IRQ_PENDING::MB3_IRQ)
        .raw_value()
        >> 0),
  ];

  pendings
}

define_mmio_register! [
    GPU_INT_ROUTING<ReadWrite<u32>@(ARM_CORE_BASE + 0x00C)>,

    CORE_MB_INT_CONTROL0<ReadWrite<u32>@(ARM_CORE_BASE + 0x050)>,
    CORE_MB_INT_CONTROL1<ReadWrite<u32>@(ARM_CORE_BASE + 0x054)>,
    CORE_MB_INT_CONTROL2<ReadWrite<u32>@(ARM_CORE_BASE + 0x058)>,
    CORE_MB_INT_CONTROL3<ReadWrite<u32>@(ARM_CORE_BASE + 0x05C)>,

    CORE0_IRQ_PENDING<ReadWrite<u32>@(ARM_CORE_BASE + 0x060)> {
      MB3_IRQ OFFSET(7)
    },
    CORE1_IRQ_PENDING<ReadWrite<u32>@(ARM_CORE_BASE + 0x064)> {
      MB3_IRQ OFFSET(7)
    },
    CORE2_IRQ_PENDING<ReadWrite<u32>@(ARM_CORE_BASE + 0x068)> {
      MB3_IRQ OFFSET(7)
    },
    CORE3_IRQ_PENDING<ReadWrite<u32>@(ARM_CORE_BASE + 0x06C)> {
      MB3_IRQ OFFSET(7)
    },

    IRQ_PENDING_B<ReadWrite<u32>@(ARM_IRQ_BASE + 0x200)>,
    IRQ_PENDING_1<ReadWrite<u32>@(ARM_IRQ_BASE + 0x204)>,
    IRQ_PENDING_2<ReadWrite<u32>@(ARM_IRQ_BASE + 0x208)>,

    FIQ_CONTROL<ReadWrite<u32>@(ARM_IRQ_BASE + 0x20C)>,

    IRQ_ENABLE_1<ReadWrite<u32>@(ARM_IRQ_BASE + 0x210)>,
    IRQ_ENABLE_2<ReadWrite<u32>@(ARM_IRQ_BASE + 0x214)>,
    IRQ_ENABLE_B<ReadWrite<u32>@(ARM_IRQ_BASE + 0x218)>,

    IRQ_DISABLE_1<ReadWrite<u32>@(ARM_IRQ_BASE + 0x21C)>,
    IRQ_DISABLE_2<ReadWrite<u32>@(ARM_IRQ_BASE + 0x220)>,
    IRQ_DISABLE_B<ReadWrite<u32>@(ARM_IRQ_BASE + 0x224)>,

    // registers allowing configuring core specific interrupts

    /// Core timer interrupts are available on each core. However, as we currently
    /// merge the interrupt sources into one pending value we will only allow to
    /// enable them on core 0 only for the time beeing
    CORE0_TIMER_IRQ<ReadWrite<u32>@(ARM_CORE_BASE + 0x040)> {
      CNTPSIRQ OFFSET(0) [
        ENABLED = 1,
        DISABLED = 0
      ],
      CNTPNSIRQ OFFSET(1) [
        ENABLED = 1,
        DISABLED = 0
      ],
      CNTHPIRQ OFFSET(2) [
        ENABLED = 1,
        DISABLED = 0
      ],
      CNTVIRQ OFFSET(3) [
        ENABLED = 1,
        DISABLED = 0
      ]
    },

    /// Mailbox interrupt control register for Core 0. Each core has 4 mailboxes
    /// thus this register can configure active interrupt for all the mailboxes
    /// for core 0. However, as we will merge the interrupt sources of all cores
    /// into one pending bit set we will allow enabling only one mailbox for each
    /// core (core id = mailbox number)
    CORE0_MAILBOX_IRQ<ReadWrite<u32>@(ARM_CORE_BASE + 0x050)> {
      MB3_IRQ OFFSET(3) [
        ENABLED = 1,
        DISABLED = 0
      ]
    },
    CORE1_MAILBOX_IRQ<ReadWrite<u32>@(ARM_CORE_BASE + 0x054)> {
      MB3_IRQ OFFSET(3) [
        ENABLED = 1,
        DISABLED = 0
      ]
    },
    CORE2_MAILBOX_IRQ<ReadWrite<u32>@(ARM_CORE_BASE + 0x058)> {
      MB3_IRQ OFFSET(3) [
        ENABLED = 1,
        DISABLED = 0
      ]
    },
    CORE3_MAILBOX_IRQ<ReadWrite<u32>@(ARM_CORE_BASE + 0x05C)> {
      MB3_IRQ OFFSET(3) [
        ENABLED = 1,
        DISABLED = 0
      ]
    },

    LOCAL_TIMER_CTRL<ReadWrite<u32>@(ARM_CORE_BASE + 0x034)> {
      RELOAD OFFSET(0) BITS(28),
      IRQ_ENABLE OFFSET(29) [
        ENABLED = 1,
        DISABLED = 0
      ]
    }
];
