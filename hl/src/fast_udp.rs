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

use w5500_ll::{Sn, SocketStatus};

use w5500_ll::Registers;
#[cfg(feature = "eha1")]
use w5500_ll::aio::Registers as RegistersAsync;

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
