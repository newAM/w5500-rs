//! Blocking implementations of the [`Registers`] trait using the
//! [`embedded-hal`] version 0.2 blocking SPI traits.
//!
//! [`embedded-hal`]: https://github.com/rust-embedded/embedded-hal
//! [`Registers`]: crate::Registers

pub use eh0 as embedded_hal;

pub mod fdm;
pub mod vdm;
pub mod vdm_infallible;
pub mod vdm_infallible_gpio;

/// Reset the W5500 using the reset pin.
///
/// This function performs the following sequence:
///
/// 1. Set the reset pin low.
/// 2. Wait 1 ms (2x longer than the minimum reset cycle time of 500 µs).
/// 3. Set the reset pin high.
/// 4. Wait 2 ms (2x longer than the maximum PLL lock time of 1 ms).
///
/// # Example
///
/// ```
/// # use ehm::eh0 as hal;
/// # let mut delay = hal::delay::NoopDelay::new();
/// # let mut reset_pin = hal::digital::Mock::new(&[
/// #    hal::digital::Transaction::set(hal::digital::State::Low),
/// #    hal::digital::Transaction::set(hal::digital::State::High),
/// # ]);
/// w5500_ll::eh0::reset(&mut reset_pin, &mut delay)?;
/// # reset_pin.done();
/// # Ok::<(), hal::MockError>(())
/// ```
pub fn reset<P, D, E>(pin: &mut P, delay: &mut D) -> Result<(), E>
where
    P: eh0::digital::v2::OutputPin<Error = E>,
    D: eh0::blocking::delay::DelayMs<u8>,
{
    pin.set_low()?;
    delay.delay_ms(1);
    pin.set_high()?;
    delay.delay_ms(2);
    Ok(())
}

/// Recommended W5500 SPI mode.
///
/// The W5500 may operate in SPI mode 0 or SPI mode 3.
pub const MODE: eh0::spi::Mode = eh0::spi::Mode {
    polarity: eh0::spi::Polarity::IdleLow,
    phase: eh0::spi::Phase::CaptureOnFirstTransition,
};
