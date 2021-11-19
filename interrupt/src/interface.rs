/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Internal interrupt interface implementation
//!
//! There are some siginificant differences between the RPi3 and RP4 with respect to the interrupt
//! configuration and available registers. The RPi4 introduces a GIC-400 interrupt controller.
//! However, the RPi4 could be run with the "legacy" interrupt controller that has been available
//! within the RPi3 already. However, running in NO-GIC mode still need to consider the different
//! register structure.
//!
//! The GIC-400 allows to route interrupts to any core. Thus the interrupt enable and pending register
//! exists 4 times (once for each core). Using the legacy mode there exist some core specific interrupts that are
//! routed to the corresponding core and core independent interrupts that are routed based on a specific
//! GPU Interrupts Routing register which core should receive those interrupts.
//!
//! For the time beeing we assume the RPi4 runs in legacy mode only. We will introduce
//! a "pi4_gic" feature in the future to also cover the GIC use case.
//!

use ruspiro_arch_aarch64::register::el1::mpidr_el1;
use ruspiro_mmio_register::define_mmio_register;

use crate::Interrupt;

#[cfg(feature = "pi3")]
const PERIPHERAL_BASE: usize = 0x0_3F00_0000;
#[cfg(feature = "pi4_low")]
const PERIPHERAL_BASE: usize = 0x0_FE00_0000;
#[cfg(feature = "pi4_high")]
const PERIPHERAL_BASE: usize = 0x4_7E00_0000;

#[cfg(feature = "pi3")]
const ARM_CORE_BASE: usize = 0x0_4000_0000;
#[cfg(feature = "pi4_low")]
const ARM_CORE_BASE: usize = 0x0_FF80_0000;
#[cfg(feature = "pi4_high")]
const ARM_CORE_BASE: usize = 0x4_C000_0000;

const ARM_IRQ_BASE: usize = PERIPHERAL_BASE + 0xB000;

pub(crate) fn initialize() {
  // disable all interrupts in all 3 banks by default
  #[cfg(feature = "pi3")]
  {
    IRQ0_DISABLE_1::Register.set(0xFFFF_FFFF);
    IRQ0_DISABLE_2::Register.set(0xFFFF_FFFF);
    IRQ0_DISABLE_B::Register.set(0xFFFF_FFFF);
  }

  #[cfg(any(feature = "pi4_low", feature = "pi4_high"))]
  {
    IRQ0_DISABLE_0::Register.set(0xFFFF_FFFF);
    IRQ0_DISABLE_1::Register.set(0xFFFF_FFFF);
    IRQ0_DISABLE_2::Register.set(0xFFFF_FFFF);
  }

  #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
  unsafe {
    asm!("dmb sy")
  };

  // set the routing of GPU interrupts to core 0
  // the register exists on PI4 as well but only configures routing for AXI_ERR_IRQ (ARM cache L2 error interrupt)
  #[cfg(feature = "pi3")]
  GPU_INT_ROUTING::Register.set(0);

  // setup IPI (inter-processor-interrupts)
  // raising IRQ only if something is written to mailbox 3 for any of the cores
  CORE0_MB_INT_CNTRL::Register.set(1 << 3);
  CORE1_MB_INT_CNTRL::Register.set(1 << 3);
  CORE2_MB_INT_CNTRL::Register.set(1 << 3);
  CORE3_MB_INT_CNTRL::Register.set(1 << 3);
}

/// globally enable ``IRQ`` interrupts to be triggered
pub(crate) fn enable_irq() {
  #[cfg(target_arch = "aarch64")]
  unsafe {
    asm!(
      "msr daifclr, #2
            isb"
    ) // as per ARM spec the ISB ensures triggering pending interrupts
  };
}

/// globally enable ``FIQ`` interrupts to be triggered
pub(crate) fn enable_fiq() {
  #[cfg(target_arch = "aarch64")]
  unsafe {
    asm!(
      "msr daifclr, #1
            isb"
    ) // as per ARM spec the ISB ensures triggering pending interrupts
  };
}

/// globally disable ``IRQ`` interrupts from beeing triggered.
pub fn disable_irq() {
  #[cfg(target_arch = "aarch64")]
  unsafe {
    asm!("msr daifset, #2")
  };
}

/// globally disable ``FIQ`` interrupts from beeing triggered.
pub fn disable_fiq() {
  #[cfg(target_arch = "aarch64")]
  unsafe {
    asm!("msr daifset, #1")
  };
}

pub(crate) fn activate(irq: Interrupt) {
  let bank = (irq as u32) >> 5;
  let enable_bit = 1 << ((irq as u32) & 0x1F);
  match bank {
    0 => {
      #[cfg(feature = "pi3")]
      IRQ0_ENABLE_1::Register.set(enable_bit);
      #[cfg(any(feature = "pi4_low", feature = "pi4_high"))]
      IRQ0_ENABLE_0::Register.set(enable_bit);
    }
    1 => {
      #[cfg(feature = "pi3")]
      IRQ0_ENABLE_2::Register.set(enable_bit);
      #[cfg(any(feature = "pi4_low", feature = "pi4_high"))]
      IRQ0_ENABLE_1::Register.set(enable_bit);
    }
    2 => {
      #[cfg(feature = "pi3")]
      IRQ0_ENABLE_B::Register.set(enable_bit);
      #[cfg(any(feature = "pi4_low", feature = "pi4_high"))]
      IRQ0_ENABLE_2::Register.set(enable_bit);
    }
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
          CORE0_MB_INT_CNTRL::Register.modify_value(CORE0_MB_INT_CNTRL::MB3_IRQ::ENABLED);
        }
        Interrupt::Core1Mailbox3 => {
          CORE1_MB_INT_CNTRL::Register.modify_value(CORE1_MB_INT_CNTRL::MB3_IRQ::ENABLED);
        }
        Interrupt::Core2Mailbox3 => {
          CORE2_MB_INT_CNTRL::Register.modify_value(CORE2_MB_INT_CNTRL::MB3_IRQ::ENABLED);
        }
        Interrupt::Core3Mailbox3 => {
          CORE3_MB_INT_CNTRL::Register.modify_value(CORE3_MB_INT_CNTRL::MB3_IRQ::ENABLED);
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
    0 => {
      #[cfg(feature = "pi3")]
      IRQ0_DISABLE_1::Register.set(disable_bit);
      #[cfg(any(feature = "pi4_low", feature = "pi4_high"))]
      IRQ0_DISABLE_0::Register.set(disable_bit);
    }
    1 => {
      #[cfg(feature = "pi3")]
      IRQ0_DISABLE_2::Register.set(disable_bit);
      #[cfg(any(feature = "pi4_low", feature = "pi4_high"))]
      IRQ0_DISABLE_1::Register.set(disable_bit);
    }
    2 => {
      #[cfg(feature = "pi3")]
      IRQ0_DISABLE_B::Register.set(disable_bit);
      #[cfg(any(feature = "pi4_low", feature = "pi4_high"))]
      IRQ0_DISABLE_2::Register.set(disable_bit);
    }
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
          CORE0_MB_INT_CNTRL::Register.modify_value(CORE0_MB_INT_CNTRL::MB3_IRQ::DISABLED);
        }
        Interrupt::Core1Mailbox3 => {
          CORE1_MB_INT_CNTRL::Register.modify_value(CORE1_MB_INT_CNTRL::MB3_IRQ::DISABLED);
        }
        Interrupt::Core2Mailbox3 => {
          CORE2_MB_INT_CNTRL::Register.modify_value(CORE2_MB_INT_CNTRL::MB3_IRQ::DISABLED);
        }
        Interrupt::Core3Mailbox3 => {
          CORE3_MB_INT_CNTRL::Register.modify_value(CORE3_MB_INT_CNTRL::MB3_IRQ::DISABLED);
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
  // get the core the interrupt has been generated
  let core = mpidr_el1::read(mpidr_el1::AFF0::Field).value();
  // use the cor specific registers to retrieve the pending interrupts
  // NOTE: the order of the register is different between PI3 and PI4 to
  // enable a stable list of pending interrupts for the caller
  match core {
    0 => {
      #[cfg(any(feature = "pi4_low", feature = "pi4_high"))]
      {
        [
          IRQ0_PENDING_0::Register.get() & IRQ0_ENABLE_0::Register.get(),
          IRQ0_PENDING_1::Register.get() & IRQ0_ENABLE_1::Register.get(),
          IRQ0_PENDING_2::Register.get() & IRQ0_ENABLE_2::Register.get(),
          CORE0_IRQ_PENDING::Register.get(),
        ]
      }
      #[cfg(feature = "pi3")]
      {
        [
          IRQ0_PENDING_1::Register.get() & IRQ0_ENABLE_1::Register.get(),
          IRQ0_PENDING_2::Register.get() & IRQ0_ENABLE_2::Register.get(),
          IRQ0_PENDING_B::Register.get() & IRQ0_ENABLE_B::Register.get(),
          CORE0_IRQ_PENDING::Register.get(),
        ]
      }
    }
    _ => unimplemented!(),
  }
}

// Define the interrupt configuration register common between Raspberry Pi3 and Pi4
define_mmio_register![
  LOCAL_TIMER_CTRL<ReadWrite<u32>@(ARM_CORE_BASE + 0x034)> {
    RELOAD OFFSET(0) BITS(28),
    ENABLE OFFSET(28),
    IRQ_ENABLE OFFSET(29) [
      ENABLED = 1,
      DISABLED = 0
    ]
  },
    /// Core timer interrupts are available on each core.
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

    /// Core timer interrupts are available on each core.
    CORE1_TIMER_IRQ<ReadWrite<u32>@(ARM_CORE_BASE + 0x044)> {
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

    /// Core timer interrupts are available on each core.
    CORE2_TIMER_IRQ<ReadWrite<u32>@(ARM_CORE_BASE + 0x048)> {
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

    /// Core timer interrupts are available on each core.
    CORE3_TIMER_IRQ<ReadWrite<u32>@(ARM_CORE_BASE + 0x04C)> {
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

  /// Core Mailbox interrupt control for ARM core 0
  CORE0_MB_INT_CNTRL<ReadWrite<u32>@(ARM_CORE_BASE + 0x050)> {
    MB0_IRQ OFFSET(0) [
      ENABLED = 1,
      DISABLED = 0
    ],
    MB1_IRQ OFFSET(1) [
        ENABLED = 1,
        DISABLED = 0
      ],
      MB2_IRQ OFFSET(2) [
        ENABLED = 1,
        DISABLED = 0
      ],
      MB3_IRQ OFFSET(3) [
        ENABLED = 1,
        DISABLED = 0
      ]
  },
  /// Core Mailbox interrupt control for ARM core 1
  CORE1_MB_INT_CNTRL<ReadWrite<u32>@(ARM_CORE_BASE + 0x054)> {
    MB0_IRQ OFFSET(0) [
      ENABLED = 1,
      DISABLED = 0
    ],
    MB1_IRQ OFFSET(1) [
        ENABLED = 1,
        DISABLED = 0
      ],
      MB2_IRQ OFFSET(2) [
        ENABLED = 1,
        DISABLED = 0
      ],
      MB3_IRQ OFFSET(3) [
        ENABLED = 1,
        DISABLED = 0
      ]
  },
  /// Core Mailbox interrupt control for ARM core 2
  CORE2_MB_INT_CNTRL<ReadWrite<u32>@(ARM_CORE_BASE + 0x058)> {
    MB0_IRQ OFFSET(0) [
      ENABLED = 1,
      DISABLED = 0
    ],
    MB1_IRQ OFFSET(1) [
        ENABLED = 1,
        DISABLED = 0
      ],
      MB2_IRQ OFFSET(2) [
        ENABLED = 1,
        DISABLED = 0
      ],
      MB3_IRQ OFFSET(3) [
        ENABLED = 1,
        DISABLED = 0
      ]
  },
  /// Core Mailbox interrupt control for ARM core 3
  CORE3_MB_INT_CNTRL<ReadWrite<u32>@(ARM_CORE_BASE + 0x05C)> {
    MB0_IRQ OFFSET(0) [
      ENABLED = 1,
      DISABLED = 0
    ],
    MB1_IRQ OFFSET(1) [
        ENABLED = 1,
        DISABLED = 0
      ],
      MB2_IRQ OFFSET(2) [
        ENABLED = 1,
        DISABLED = 0
      ],
      MB3_IRQ OFFSET(3) [
        ENABLED = 1,
        DISABLED = 0
      ]
  },
    /// ARM core interrupt source for core 0
  CORE0_IRQ_PENDING<ReadWrite<u32>@(ARM_CORE_BASE + 0x060)> {
    MB3_IRQ OFFSET(7)
  },
  /// ARM core interrupt source for core 1
  CORE1_IRQ_PENDING<ReadWrite<u32>@(ARM_CORE_BASE + 0x064)> {
    MB3_IRQ OFFSET(7)
  },
  /// ARM core interrupt source for core 2
  CORE2_IRQ_PENDING<ReadWrite<u32>@(ARM_CORE_BASE + 0x068)> {
    MB3_IRQ OFFSET(7)
  },
  /// ARM core interrupt source for core 3
  CORE3_IRQ_PENDING<ReadWrite<u32>@(ARM_CORE_BASE + 0x06C)> {
    MB3_IRQ OFFSET(7)
  }
];

// Define the interrupt configuration register for the Raspberry Pi3
#[cfg(feature = "pi3")]
define_mmio_register! [
    GPU_INT_ROUTING<ReadWrite<u32>@(ARM_CORE_BASE + 0x00C)>,

    /// Basic pending interrupts 7..19, 53..57 and 62 in bits 10..20
    IRQ0_PENDING_B<ReadWrite<u32>@(ARM_IRQ_BASE + 0x200)> {
      ARM_TIMER_IRQ OFFSET(0),
      ARM_MAILBOX_IRQ0 OFFSET(1),
      ARM_DOORBELL_IRQ0 OFFSET(2),
      ARM_DOORBELL_IRQ1 OFFSET(3),
      VPU_C0_C1_HALT OFFSET(4),
      VPU_C1_HALT OFFSET(5),
      ARM_ILLEGAL_ACCESS1 OFFSET(6),
      ARM_ILLEGAL_ACCESS0 OFFSET(7),
      PENDING2 OFFSET(8),
      PENDING1 OFFSET(9)
    },
    /// Pending interrupts 0 .. 31
    IRQ0_PENDING_1<ReadWrite<u32>@(ARM_IRQ_BASE + 0x204)>,
    /// Pending interrupts 32 .. 63
    IRQ0_PENDING_2<ReadWrite<u32>@(ARM_IRQ_BASE + 0x208)>,

    FIQ_CONTROL<ReadWrite<u32>@(ARM_IRQ_BASE + 0x20C)>,

    IRQ0_ENABLE_1<ReadWrite<u32>@(ARM_IRQ_BASE + 0x210)>,
    IRQ0_ENABLE_2<ReadWrite<u32>@(ARM_IRQ_BASE + 0x214)>,
    IRQ0_ENABLE_B<ReadWrite<u32>@(ARM_IRQ_BASE + 0x218)>,

    IRQ0_DISABLE_1<ReadWrite<u32>@(ARM_IRQ_BASE + 0x21C)>,
    IRQ0_DISABLE_2<ReadWrite<u32>@(ARM_IRQ_BASE + 0x220)>,
    IRQ0_DISABLE_B<ReadWrite<u32>@(ARM_IRQ_BASE + 0x224)>
];

// define the interrupt configuration registers for the Raspberry Pi4 (legacy mode)
#[cfg(any(feature = "pi4_low", feature = "pi4_high"))]
define_mmio_register! [
  /// VC pending interrupts 0 .. 31 of core 0
  IRQ0_PENDING_0<ReadOnly<u32>@(ARM_IRQ_BASE + 0x200)>,
  /// VC pending interrupts 32 .. 63 of core 0
  IRQ0_PENDING_1<ReadOnly<u32>@(ARM_IRQ_BASE + 0x204)>,
  /// VC pending interrupts 64 .. 79
  IRQ0_PENDING_2<ReadOnly<u32>@(ARM_IRQ_BASE + 0x208)> {
    ARM_TIMER_IRQ OFFSET(0),
    ARM_MAILBOX_IRQ0 OFFSET(1),
    ARM_DOORBELL_IRQ0 OFFSET(2),
    ARM_DOORBELL_IRQ1 OFFSET(3),
    VPU_C0_C1_HALT OFFSET(4),
    VPU_C1_HALT OFFSET(5),
    ARM_ADDR_ERROR OFFSET(6),
    ARM_AXI_ERROR OFFSET(7),
    /// Software interrupts
    SW_TRIG_INT OFFSET(8) BITS(8),
    PENDING2 OFFSET(24),
    PENDING1 OFFSET(25),
    ARM_IRQ OFFSET(31)
  },
  /// set bits to enable VC interrupts 0..31
  IRQ0_ENABLE_0<ReadWrite<u32>@(ARM_IRQ_BASE + 0x210)>,
  /// set bits to enable VC interrupts 32..63
  IRQ0_ENABLE_1<ReadWrite<u32>@(ARM_IRQ_BASE + 0x214)>,
  /// set bits to enable VC interrupts 64..79
  IRQ0_ENABLE_2<ReadWrite<u32>@(ARM_IRQ_BASE + 0x218)>,
  /// set bits to disable VC interrupts 0..31
  IRQ0_DISABLE_0<ReadWrite<u32>@(ARM_IRQ_BASE + 0x220)>,
  /// set bits to disable VC interrupts 32..63
  IRQ0_DISABLE_1<ReadWrite<u32>@(ARM_IRQ_BASE + 0x224)>,
  /// set bits to disable VC interrupts 64..79
  IRQ0_DISABLE_2<ReadWrite<u32>@(ARM_IRQ_BASE + 0x228)>
];
