//! Blocking implementations of the [`Registers`] trait using the
//! [`embedded-hal`] blocking SPI traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//! [`Registers`]: crate::Registers

pub mod fdm;
pub mod vdm;
