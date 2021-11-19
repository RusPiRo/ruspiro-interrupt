/***********************************************************************************************************************
 * Copyright (c) 2019 by the authors
 *
 * Author: André Borrmann
 * License: Apache License 2.0
 **********************************************************************************************************************/

//! # Interrupt Types
//!
//! Defining the different possible interrupts of the Raspberry Pi a handler could be registered for.
//!

/// The list of available interrupts on Raspberry Pi 3.
/// Note: Even if it is possible to register an interrupt handler for them the behaviour might be untested/undefined.
/// Please read the corresponding specs for the different interrupts to understand how to acknowledge them inside the
/// individual handler implementation.
///
#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum Interrupt {
  // IRQ's appearing in the GPU pending registers
  // IRQ 0 - 31 (only the IRQ's that could be registered)
  SystemTimer1 = 1,
  SystemTimer3 = 3,
  Isp = 8,
  Usb = 9, // Synopsys DesignWare Hi-Speed USB 2.0 OTG controller IRQ. Also available as IRQ 75 in basic pending
  CoreSync0 = 12,
  CoreSync1 = 13,
  CoreSync2 = 14,
  CoreSync3 = 15,
  /*
  DMA0 = 16,
  DMA1 = 17,
  DMA2 = 18,
  DMA3 = 19,
  DMA4 = 20,
  DMA5 = 21,
  DMA6 = 22,
  DMA7_8 = 23,
  DMA9_10 = 24,
  DMA11 = 25,
  DMA12 = 26,
  DMA13 = 27,
  DMA14 = 28,
  */
  Aux = 29,
  // ARM
  Arm = 30,
  // GPU-DMA
  GpuDma = 31,

  // IRQ 32 - 63
  /*HostPort        = 32, // HDMI CEC
  VideoScaler     = 33, // HVS
  Ccp2Tx          = 34, // RPIVID
  Sdc             = 35,
  Dsi0            = 36,
  Ave             = 37, // Pixel Valve2
  Cam0            = 38,
  Cam1            = 39,
  Hdmi0           = 40,
  Hdmi1           = 41,
  PixelValve3     = 42,
  I2cSpi          = 43,
  Dsi1            = 44,
  Pwa0            = 45, // PixelValve0
  Pwa1            = 46, // PixelValve 1 & 4
  Cpr             = 47,
  Smi             = 48,
  */
  GpioBank0 = 49, // GPIO Bank 0
  GpioBank1 = 50,
  GpioBank2 = 51, // Not existend at BCM2837???
  GpioBank3 = 52,
  I2c = 53,    // also available as IRQ 79 in basic pending
  Spi = 54,    // also available as IRQ 80 in basic pending
  I2sPcm = 55, // also available as IRQ 81 in basic pending
  Sdio = 56,   // also available as IRQ 82 in basic pending
  Pl011 = 57,  // also avialable as IRQ 83 in basic pending
  // Eth_PCIe  = 58,

  // IRQ 64 - 95 - Bank Basic
  ArmTimer = 64,
  ArmMailbox = 65,
  ArmDoorbell0 = 66,
  ArmDoorbell1 = 67,
  ArmGpu0Halted = 68,
  ArmGpu1Halted = 69,
  ArmIllegalType1 = 70,
  ArmIllegalType0 = 71,
  ArmPending1 = 72,
  ArmPending2 = 73,
  // IRQ 96 - 127
  // ARM Core specific interrupts
  CntPsIrq = 96,
  CntPnsIrq = 97,
  CntHpIrq = 98,
  CntVIrq = 99,
  Core0Mailbox3 = 100,
  Core1Mailbox3 = 101,
  Core2Mailbox3 = 102,
  Core3Mailbox3 = 103,
  CoreGPU = 104,
  //CorePMU = 105,
  //CoreAxi = 106,
  LocalTimer = 107,
}
