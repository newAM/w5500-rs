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

use w5500_ll::{Protocol, Sn, SocketCommand, SocketMode, SocketStatus, net::SocketAddrV4};

use crate::Error;
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
    async(
        feature = "eha1",
        idents(
            Registers(async = "RegistersAsync"),
            yield_once(async = "yield_once_async")
        )
    )
)]
pub trait FastUdp: Registers {
    /// Open `socket` as a UDP socket bound to `port`.
    ///
    /// Uses the W5500 hardware socket engine: `Sn_MR` protocol field is set to
    /// UDP (W5500 datasheet section 5.2). MACRAW is never used.
    ///
    /// # Blocking
    ///
    /// Spins on `Sn_SR` twice, yielding to the executor between polls. This is
    /// an initialization-path cost only; the datasheet guarantees the status
    /// changes after CLOSE and after OPEN, so neither loop is unbounded in
    /// practice. It is a busy-yield: it does not block other tasks, but it does
    /// not idle the CPU either.
    async fn udp_bind(&mut self, socket: Sn, port: u16) -> Result<(), Self::Error> {
        self.set_sn_cr(socket, SocketCommand::Close).await?;
        while self.sn_sr(socket).await? != Ok(SocketStatus::Closed) {
            yield_once().await;
        }

        self.set_sn_port(socket, port).await?;
        const UDP_MODE: SocketMode = SocketMode::DEFAULT.set_protocol(Protocol::Udp);
        self.set_sn_mr(socket, UDP_MODE).await?;
        self.set_sn_cr(socket, SocketCommand::Open).await?;
        while self.sn_sr(socket).await? != Ok(SocketStatus::Udp) {
            yield_once().await;
        }
        Ok(())
    }

    /// Open `socket` as a UDP socket bound to `port`, with its destination
    /// fixed to `peer`.
    ///
    /// For the single-fixed-peer case. After this call use
    /// [`FastUdp::udp_send`] or [`FastUdp::udp_send_exact`] in the hot loop:
    /// they never rewrite `Sn_DIPR`/`Sn_DPORT`, saving two SPI transactions per
    /// send compared with [`FastUdp::udp_send_to`].
    async fn udp_bind_to_peer(
        &mut self,
        socket: Sn,
        port: u16,
        peer: &SocketAddrV4,
    ) -> Result<(), Self::Error> {
        self.udp_bind(socket, port).await?;
        self.set_sn_dest(socket, peer).await
    }

    /// Receive one datagram into `frame`, header and payload in a single SPI
    /// transaction.
    ///
    /// The W5500 socket RX buffer is contiguous with auto-increment (W5500
    /// datasheet section 4.2), so the 8-byte header and the payload are read
    /// together and the header is deserialized in place. Combined with the
    /// adjacent-register read of `Sn_RX_RSR`/`Sn_RX_RD`, a receive costs **four**
    /// SPI transactions where the generic path costs six.
    ///
    /// # Errors
    ///
    /// - [`Error::WouldBlock`] if no complete datagram is buffered. Cheap and
    ///   non-fatal: intended to be called at a fixed cadence. There is no
    ///   internal retry loop.
    /// - [`Error::UnexpectedLength`] if the datagram payload is not exactly
    ///   `UdpFrame::<FRAME_LEN>::PAYLOAD_LEN` bytes. The datagram is discarded
    ///   and the socket stays usable — it is never truncated silently.
    /// - [`Error::Other`] for register IO failures.
    ///
    async fn udp_recv_exact<const FRAME_LEN: usize>(
        &mut self,
        socket: Sn,
        frame: &mut UdpFrame<FRAME_LEN>,
    ) -> Result<(), Error<Self::Error>> {
        const { assert!(FRAME_LEN > UDP_FRAME_HEADER_LEN) };

        let expected_payload_len: u16 = (FRAME_LEN - UDP_FRAME_HEADER_LEN) as u16;
        let frame_len: u16 = FRAME_LEN as u16;

        let pointers = self.sn_rx_ptrs(socket).await?;
        if pointers.rsr < UdpHeader::LEN {
            return Err(Error::WouldBlock);
        }


        if pointers.rsr < frame_len {
            return self
                .recv_resync_short(socket, pointers.rsr, pointers.rd, expected_payload_len)
                .await;
        }

        self.sn_rx_buf(socket, pointers.rd, frame.buffer_mut())
            .await?;

        let received_payload_len: u16 = frame.header_payload_len();
        let consumed: u16 = UdpHeader::LEN.wrapping_add(received_payload_len);
        self.set_sn_rx_rd(socket, pointers.rd.wrapping_add(consumed))
            .await?;
        self.set_sn_cr(socket, SocketCommand::Recv).await?;

        if received_payload_len != expected_payload_len {
            return Err(Error::UnexpectedLength {
                expected: expected_payload_len,
                received: received_payload_len,
            });
        }
        Ok(())
    }

    /// Cold path: fewer bytes buffered than a full frame.
    ///
    /// `#[doc(hidden)]` is expressed as `cfg_attr(doc, ..)` here: written as a
    /// bare `#[doc(hidden)]`, `maybe-async-cfg`'s default `doctests` feature
    /// misparses it as a `#[doc = "..."]` line following the doc comment
    /// above and fails to compile. Gating it behind `cfg_attr(doc, ..)` keeps
    /// the attribute's path as `cfg_attr`, which that pass does not touch, and
    /// is equivalent for rustdoc's purposes: `doc(hidden)` only affects
    /// generated documentation, and `cfg(doc)` is set exactly when rustdoc is
    /// building it.
    #[cfg_attr(doc, doc(hidden))]
    async fn recv_resync_short(
        &mut self,
        _socket: Sn,
        _received_size: u16,
        _read_pointer: u16,
        _expected_payload_len: u16,
    ) -> Result<(), Error<Self::Error>> {
        Err(Error::WouldBlock)
    }
}

/// No-op in the blocking build: the sync spin loop busy-waits without
/// yielding to anything.
///
/// Paired with [`yield_once_async`] via `maybe-async-cfg`'s `idents` renaming
/// on [`FastUdp::udp_bind`], so the sync half of the trait never references
/// the async-only yield helper.
fn yield_once() {}

/// Yield to the executor once, re-arming immediately.
///
/// Executor-agnostic: no `embassy-time` or other runtime dependency. Used only
/// on the initialization path in [`FastUdp::udp_bind`].
#[cfg(feature = "eha1")]
async fn yield_once_async() {
    core::future::poll_fn(|context| {
        context.waker().wake_by_ref();
        core::task::Poll::Pending
    })
    .await
}

#[maybe_async_cfg::maybe(
    sync(keep_self),
    async(feature = "eha1", idents(Registers(async = "RegistersAsync"), FastUdp(async = "FastUdpAsync")))
)]
impl<RegisterAccess> FastUdp for RegisterAccess where RegisterAccess: Registers {}
