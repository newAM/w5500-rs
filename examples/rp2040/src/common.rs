//! Shared board setup for the W5500 on SPI0.
//!
//! The MCP251xFD controllers on this board are on SPI1; nothing here touches
//! them.

use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice as SharedSpiDevice;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{SPI0, USB};
use embassy_rp::spi::{Async, Config as SpiConfig, Phase, Polarity, Spi};
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::Timer;
use static_cell::StaticCell;
use w5500_ll::eh1::embedded_hal_async::spi::{Operation, SpiDevice as AsyncSpiDevice};
use w5500_ll::spi::{AccessMode, vdm_header};
use w5500_ll::{
    Sn,
    aio::Registers,
    net::{Eui48Addr, Ipv4Addr, SocketAddrV4},
};

pub type Bus = Mutex<NoopRawMutex, Spi<'static, SPI0, Async>>;
pub type Device = SharedSpiDevice<'static, NoopRawMutex, Spi<'static, SPI0, Async>, Output<'static>>;
pub type UsbDriver = Driver<'static, USB>;

/// The concrete error type a register call on a [`Device`] can return.
///
/// Spelled out so binary bodies can log the discriminant instead of
/// `unwrap()`-ing: without a debug probe a panic is an invisible hang.
pub type SpiError = embassy_embedded_hal::shared_bus::SpiDeviceError<
    embassy_rp::spi::Error,
    core::convert::Infallible,
>;

/// Async variable-data-length W5500 driver over a [`Device`].
///
/// `w5500_ll::eh1::vdm::W5500::new` cannot be used here: its constructor is
/// bounded on `embedded_hal::spi::SpiDevice` (the *blocking* trait), even
/// though its `w5500_ll::aio::Registers` impl only needs
/// `embedded_hal_async::spi::SpiDevice`. `Device` -- an
/// `embassy_embedded_hal` shared-bus device built on async DMA SPI -- only
/// ever implements the async trait, so that constructor bound can never be
/// satisfied for it. This reimplements the same variable-data-length framing
/// (`w5500_ll::spi::vdm_header`, matching `w5500_ll::eh1::vdm`'s async
/// `Registers` impl byte for byte) directly against the async trait instead.
pub struct W5500Device {
    spi: Device,
}

impl W5500Device {
    /// Wraps a [`Device`] for W5500 register access.
    pub fn new(spi: Device) -> Self {
        W5500Device { spi }
    }
}

impl Registers for W5500Device {
    type Error = SpiError;

    async fn read(&mut self, address: u16, block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Read);
        let mut operations = [Operation::Write(&header), Operation::Read(data)];
        AsyncSpiDevice::transaction(&mut self.spi, &mut operations).await
    }

    async fn write(&mut self, address: u16, block: u8, data: &[u8]) -> Result<(), Self::Error> {
        let header = vdm_header(address, block, AccessMode::Write);
        let mut operations = [Operation::Write(&header), Operation::Write(data)];
        AsyncSpiDevice::transaction(&mut self.spi, &mut operations).await
    }
}

static SPI_BUS: StaticCell<Bus> = StaticCell::new();

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

/// Requested SPI clock.
///
/// **The RP2040 cannot produce 50 MHz.** Its SPI divider yields
/// `clk_peri / (CPSDVSR * (1 + SCR))` with `CPSDVSR` even, so from a 125 MHz
/// `clk_peri` the reachable rates near the top are 62.5 MHz and 31.25 MHz with
/// nothing in between. `embassy-rp` rounds a request down, so asking for 50 MHz
/// silently gives 31.25 MHz.
///
/// 31.25 MHz is chosen deliberately as the conservative default: it is well
/// inside the W5500's SPI rating and leaves margin on a board whose trace
/// lengths have not been characterised. Run `spiclock` to measure what the
/// peripheral actually produced and to find the highest rate this board stays
/// clean at, then revisit this constant.
pub const SPI_FREQUENCY_HZ: u32 = 31_250_000;

pub const MAC: Eui48Addr = Eui48Addr::new(0x02, 0x00, 0x00, 0x00, 0x00, 0x10);
pub const DEVICE_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 0, 10);
pub const SUBNET: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 0);
pub const GATEWAY: Ipv4Addr = Ipv4Addr::new(192, 168, 0, 1);
pub const PEER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 1), 49200);
pub const DEVICE_PORT: u16 = 49200;
pub const SOCKET: Sn = Sn::Sn6;

/// Inbound payload length. Anything else is a protocol error.
pub const PAYLOAD_LEN: usize = 180;
/// Frame length: the 8-byte W5500 receive header plus the payload.
pub const FRAME_LEN: usize = 188;
/// Outbound payload: 8 × f32 motor commands.
pub const REPLY_LEN: usize = 32;

/// The f32 pattern the host places at [`payload::PATTERN_OFFSET`].
///
/// Deliberately asymmetric byte-wise so a big-endian misread cannot coincide
/// with the expected value. `1.0f32` is `0x3F800000`: little-endian
/// `[00, 00, 80, 3F]`, big-endian `[3F, 80, 00, 00]`.
pub const ENDIAN_PATTERN: [f32; 8] = [1.0, -2.0, 0.5, 3.25, -0.125, 100.0, 0.0, -7.5];

/// Inbound payload layout. All fields little-endian.
pub mod payload {
    pub const SEQUENCE_OFFSET: usize = 0;
    pub const TIMESTAMP_OFFSET: usize = 4;
    pub const PATTERN_OFFSET: usize = 12;
    pub const FILLER_OFFSET: usize = 44;
    pub const FILLER_BYTE: u8 = 0xA5;

    /// Sequence number, or `None` if the payload is too short.
    pub fn read_sequence(payload: &[u8]) -> Option<u32> {
        let bytes = payload.get(SEQUENCE_OFFSET..SEQUENCE_OFFSET + 4)?;
        let word: [u8; 4] = bytes.try_into().ok()?;
        Some(u32::from_le_bytes(word))
    }

    /// Host send timestamp in microseconds, or `None` if the payload is short.
    pub fn read_timestamp(payload: &[u8]) -> Option<u64> {
        let bytes = payload.get(TIMESTAMP_OFFSET..TIMESTAMP_OFFSET + 8)?;
        let word: [u8; 8] = bytes.try_into().ok()?;
        Some(u64::from_le_bytes(word))
    }

    /// The 8-f32 pattern, or `None` if the payload is too short.
    pub fn read_pattern(payload: &[u8]) -> Option<[f32; 8]> {
        let bytes = payload.get(PATTERN_OFFSET..PATTERN_OFFSET + 32)?;
        let mut pattern: [f32; 8] = [0.0; 8];
        for (index, slot) in pattern.iter_mut().enumerate() {
            let word: [u8; 4] = bytes.get(index * 4..index * 4 + 4)?.try_into().ok()?;
            *slot = f32::from_le_bytes(word);
        }
        Some(pattern)
    }
}

/// Encode 8 motor commands as 32 little-endian bytes.
pub fn encode_reply(commands: &[f32; 8]) -> [u8; REPLY_LEN] {
    let mut reply: [u8; REPLY_LEN] = [0; REPLY_LEN];
    for (index, command) in commands.iter().enumerate() {
        reply[index * 4..index * 4 + 4].copy_from_slice(&command.to_le_bytes());
    }
    reply
}

/// Brings up SPI0 (SCK=GP18, MOSI=GP19, MISO=GP20) with the W5500's chip select
/// on GP17, plus the USB peripheral the log output leaves through.
///
/// The bus handle is returned so `spiclock` can re-clock at runtime via
/// `Spi::set_config`.
///
/// Pin assignment note: the RP2040 pin mux fixes SPI0 TX to GP19 and SPI0 RX to
/// GP20. A board wired the other way round cannot use the SPI0 peripheral at
/// all. `identify` is what confirms this assignment is right.
pub fn init_board() -> (Device, UsbDriver, &'static Bus) {
    let peripherals = embassy_rp::init(Default::default());

    let mut spi_config = SpiConfig::default();
    spi_config.frequency = SPI_FREQUENCY_HZ;
    // The W5500 uses SPI mode 0 (datasheet section 3: sampled on the rising
    // edge, shifted on the falling edge). These match `SpiConfig::default()`,
    // but a hard chip requirement should not rest on an upstream default
    // staying put.
    spi_config.phase = Phase::CaptureOnFirstTransition;
    spi_config.polarity = Polarity::IdleLow;

    let spi = Spi::new(
        peripherals.SPI0,
        peripherals.PIN_18, // SCK
        peripherals.PIN_19, // MOSI (SPI0 TX)
        peripherals.PIN_20, // MISO (SPI0 RX)
        peripherals.DMA_CH0,
        peripherals.DMA_CH1,
        spi_config,
    );
    let bus: &'static Bus = SPI_BUS.init(Mutex::new(spi));
    let chip_select = Output::new(peripherals.PIN_17, Level::High);
    let device = SharedSpiDevice::new(bus, chip_select);

    (device, Driver::new(peripherals.USB, Irqs), bus)
}

/// Runs the USB CDC-ACM serial device that carries every `log` line.
///
/// Must be spawned before the first log call. The logger writes into a
/// non-blocking 1 KiB pipe, so output produced while no terminal has the port
/// open is dropped rather than stalling -- which is why each binary repeats its
/// sweep instead of reporting once.
#[embassy_executor::task]
pub async fn logger_task(driver: UsbDriver) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

/// Gives the host a moment to enumerate the CDC device before the first log
/// line, so a terminal already open across a re-flash catches the first pass.
pub async fn wait_for_host() {
    Timer::after_secs(2).await;
}
