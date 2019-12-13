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
#[derive(Copy, Clone)]
pub enum Interrupt {
    // IRQ's appearing in the GPU pending register 1 and 2
    // IRQ 0 - 31 / Bank 1 (only the IRQ's that could be registered)
    SystemTimer1 = 1,
    SystemTimer3 = 3,
    Isp = 8,
    // USB
    Usb = 9, // Synopsys DesignWare Hi-Speed USB 2.0 OTG controller IRQ. Also available as IRQ 75 in basic pending
    CoreSync0 = 12,
    CoreSync1 = 13,
    CoreSync2 = 14,
    CoreSync3 = 15,
    Aux = 29,
    // ARM
    Arm = 30,
    // GPU-DMA
    GpuDma = 31,

    // IRQ 32 - 63 / Bank 2
    /*HostPort        = 32,
    VideoScaler     = 33,
    Ccp2Tx          = 34,
    Sdc             = 35,
    Dsi0            = 36,
    Ave             = 37,
    Cam0            = 38,
    Cam1            = 39,
    Hdmi0           = 40,
    Hdmi1           = 41,
    PixelValve1     = 42,
    I2cSpi          = 43,
    Dsi1            = 44,
    Pwa0            = 45,
    Pwa1            = 46,
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
    ArmTimer = 64,
    ArmMailbox = 65,
    ArmDoorbell0 = 66,
    ArmDoorbell1 = 67,
    ArmGpu0Halted = 68,
    ArmGpu1halted = 69,
    ArmIllegalType1 = 70,
    ArmIllegalType0 = 71,
    ArmPending1 = 72,
    ArmPending2 = 73,
}
