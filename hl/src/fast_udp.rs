//! Fixed-size UDP fast path over the W5500 hardware socket engine.
//!
//! This module is an addition to this fork; it is not part of upstream
//! `w5500-hl`. It exists alongside [`crate::Udp`] rather than replacing it, so
//! the fork stays re-syncable with upstream.
//!
//! Sync and async implementations are generated from this one source by
//! `maybe-async-cfg`, so they cannot drift: [`FastUdp`] over
//! [`w5500_ll::Registers`], and `FastUdpAsync` over
//! [`w5500_ll::aio::Registers`] behind the `eha1` feature.

use w5500_ll::{Sn, SocketStatus, net::SocketAddrV4};

use crate::udp::UdpHeader;
use w5500_ll::Registers;
#[cfg(feature = "eha1")]
use w5500_ll::aio::Registers as RegistersAsync;

/// Number of bytes in the W5500 UDP receive header.
///
/// W5500 datasheet section 4.2 (`Sn_RX_BUF`): a received UDP datagram is
/// prefixed in the socket RX buffer by 4 bytes origin IP, 2 bytes origin port
/// and 2 bytes length, all big-endian. This is **not** the 8-byte wire UDP
/// header.
pub const UDP_FRAME_HEADER_LEN: usize = UdpHeader::LEN_USIZE;

/// A W5500 UDP receive frame: the 8-byte header followed by exactly
/// `FRAME_LEN - UDP_FRAME_HEADER_LEN` payload bytes.
///
/// One contiguous buffer, so [`FastUdp::udp_recv_exact`] can fill the header
/// and the payload in a single SPI transaction and the payload is never copied
/// out.
///
/// # Limitations
///
/// - `FRAME_LEN` is the **frame** length, header included, not the payload
///   length. `UdpFrame<188>` carries a 180-byte payload. Expressing the buffer
///   as `[u8; UDP_FRAME_HEADER_LEN + PAYLOAD_LEN]` would need the unstable
///   `generic_const_exprs` feature.
/// - [`Self::payload`] returns a slice rather than a fixed-size array for the
///   same reason. Its length is nonetheless statically fixed; use
///   `frame.payload().first_chunk::<N>()` to recover a typed array.
/// - `FRAME_LEN` must be greater than [`UDP_FRAME_HEADER_LEN`]. This is a
///   compile-time error, not a runtime panic.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UdpFrame<const FRAME_LEN: usize> {
    buffer: [u8; FRAME_LEN],
}

impl<const FRAME_LEN: usize> Default for UdpFrame<FRAME_LEN> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const FRAME_LEN: usize> UdpFrame<FRAME_LEN> {
    /// Payload length carried by a frame of this size.
    pub const PAYLOAD_LEN: usize = FRAME_LEN - UDP_FRAME_HEADER_LEN;

    /// Create a frame buffer.
    ///
    /// Allocate one outside the hot loop and reuse it: this zeroes the buffer,
    /// and a receive overwrites every byte it reports.
    pub const fn new() -> Self {
        const { assert!(FRAME_LEN > UDP_FRAME_HEADER_LEN) };
        Self {
            buffer: [0; FRAME_LEN],
        }
    }

    /// The datagram payload, without the W5500 header.
    ///
    /// Always exactly [`Self::PAYLOAD_LEN`] bytes.
    pub fn payload(&self) -> &[u8] {
        &self.buffer[UDP_FRAME_HEADER_LEN..]
    }

    /// The datagram payload, mutably, for decoding in place.
    pub fn payload_mut(&mut self) -> &mut [u8] {
        &mut self.buffer[UDP_FRAME_HEADER_LEN..]
    }

    /// Source address and port, decoded from the header.
    ///
    /// Big-endian, per W5500 datasheet section 4.2.
    pub fn origin(&self) -> SocketAddrV4 {
        let mut header_bytes: [u8; UDP_FRAME_HEADER_LEN] = [0; UDP_FRAME_HEADER_LEN];
        header_bytes.copy_from_slice(&self.buffer[..UDP_FRAME_HEADER_LEN]);
        UdpHeader::deser(header_bytes).origin
    }

    /// Payload length the header claims, in bytes.
    pub(crate) fn header_payload_len(&self) -> u16 {
        let mut header_bytes: [u8; UDP_FRAME_HEADER_LEN] = [0; UDP_FRAME_HEADER_LEN];
        header_bytes.copy_from_slice(&self.buffer[..UDP_FRAME_HEADER_LEN]);
        UdpHeader::deser(header_bytes).len
    }

    /// The whole frame buffer, for the receive path to fill.
    pub(crate) fn buffer_mut(&mut self) -> &mut [u8; FRAME_LEN] {
        &mut self.buffer
    }

    /// Test-only mutable access to the whole frame.
    #[doc(hidden)]
    pub fn buffer_for_test(&mut self) -> &mut [u8; FRAME_LEN] {
        &mut self.buffer
    }
}

#[maybe_async_cfg::maybe(
    sync(keep_self),
    async(feature = "eha1", idents(Registers(async = "RegistersAsync")))
)]
pub trait FastUdp: Registers {
    /// Provisional method proving the code-generation mechanism.
    ///
    /// Replaced by the real API in later tasks.
    async fn udp_socket_status(&mut self, socket: Sn) -> Result<Result<SocketStatus, u8>, Self::Error> {
        self.sn_sr(socket).await
    }
}

#[maybe_async_cfg::maybe(
    sync(keep_self),
    async(feature = "eha1", idents(Registers(async = "RegistersAsync"), FastUdp(async = "FastUdpAsync")))
)]
impl<RegisterAccess> FastUdp for RegisterAccess where RegisterAccess: Registers {}
