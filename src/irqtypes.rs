/* 
 * irqtypes.rs Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: ???
 */
//! # IRQ Types
//! Defining the different possible IRQ's that can a handler could be registered for

/// The list of available interrupts on Raspberry Pi 3. Not for all of them an interrupt handler can be successfully
/// implemented.
///
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Interrupt {     
    // IRQ's appearing in the GPU pending register 1 and 2
    // IRQ 0 - 31 / Bank 1 (only the IRQ's that could be registered)
    SystemTimer1    = 1,
    SystemTimer3    = 3,
    // Codec IRQ's
    Codec0          = 4,
    Codec1          = 5,
    Codec2          = 6,
    // JPEG
    Jpeg            = 7, // also avilable as IRQ 74 in basic pending
    // ISP
    Isp             = 8,
    // USB
    Usb             = 9, // Synopsys DesignWare Hi-Speed USB 2.0 OTG controller IRQ. Also available as IRQ 75 in basic pending
    // 3D
    ThreeD          = 10, // also available as IRQ 75 in basic pending
    // transponder
    Transponder     = 11,
    // multi core sync
    CoreSync0       = 12,
    CoreSync1       = 13,
    CoreSync2       = 14,
    CoreSync3       = 15,
    // DMA
    Dma0            = 16,
    Dma1            = 17,
    Dma2            = 18,
    Dma3            = 19,
    Dma4            = 20,
    Dma5            = 21,
    Dma6            = 22,
    Dma7            = 23,
    Dma8            = 24,
    Dma9            = 25,
    Dma10           = 26,
    Dma11           = 27,
    Dma12           = 28,
    // UART1, SPI1, SPI2
    Aux             = 29,
    // ARM
    Arm             = 30,
    // GPU-DMA
    GpuDma          = 31,

    // IRQ 32 - 63 / Bank 2
    HostPort        = 32,
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
    GpioBank0       = 49, // GPIO Bank 0
    GpioBank1       = 50,
    GpioBank2       = 51, // Not existend at BCM2837???
    GpioBank3       = 52,
    I2c             = 53, // also available as IRQ 79 in basic pending
    Spi             = 54, // also available as IRQ 80 in basic pending
    I2sPcm          = 55, // also available as IRQ 81 in basic pending
    Sdio            = 56, // also available as IRQ 82 in basic pending
    Pl011           = 57, // also avialable as IRQ 83 in basic pending
    SlimBus         = 58,
    Vec             = 59,
    Cpg             = 60,
    Rng             = 61,
    Sdhci           = 62, // also available as IRQ 84 in basic pending
    AvsPmon         = 63,

    // IRQ 64 - 84 / bank basic pending
    ArmTimer        = 64,
    ArmMailbox      = 65,
    ArmDoorbell0    = 66,
    ArmDoorbell1    = 67,
    ArmGpu0Halted   = 68,
    ArmGpu1halted   = 69,
    ArmIllegalType1 = 70,
    ArmIllegalType0 = 71,
    ArmPending1     = 72,
    ArmPending2     = 73,
}

impl Interrupt {
    pub fn from_u8(num: u8) -> Option<Interrupt> {
        match num {
            49 => Some(Interrupt::GpioBank0),
            50 => Some(Interrupt::GpioBank1),
            51 => Some(Interrupt::GpioBank2),
            52 => Some(Interrupt::GpioBank3),
            64 => Some(Interrupt::ArmTimer),
            65 => Some(Interrupt::ArmMailbox),
            66 => Some(Interrupt::ArmDoorbell0),
            67 => Some(Interrupt::ArmDoorbell1),
            _ => None
        }
    }
}