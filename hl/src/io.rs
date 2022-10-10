//! Socket buffer IO traits.

use crate::Error;

/// Enumeration of all possible methods to seek the W5500 socket buffers.
///
/// This is designed to be similar to [`std::io::SeekFrom`].
///
/// [`std::io::SeekFrom`]: https://doc.rust-lang.org/std/io/enum.SeekFrom.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SeekFrom {
    /// Sets the offset to the provided number of bytes.
    Start(u16),
    /// Sets the offset to the end plus the specified number of bytes.
    End(i16),
    /// Sets the offset to the current position plus the specified number of bytes.
    Current(i16),
}

// TODO: use wrapping_add_signed when stabilized
// https://github.com/rust-lang/rust/issues/87840
// https://github.com/rust-lang/rust/blob/21b0325c68421b00c6c91055ac330bd5ffe1ea6b/library/core/src/num/uint_macros.rs#L1205
fn wrapping_add_signed(ptr: u16, offset: i16) -> u16 {
    ptr.wrapping_add(offset as u16)
}

impl SeekFrom {
    /// Calculate the next value of `ptr` for the given seek method.
    #[doc(hidden)]
    pub fn new_ptr<E>(self, ptr: u16, head: u16, tail: u16) -> Result<u16, Error<E>> {
        match self {
            SeekFrom::Start(offset) => {
                if offset > tail.wrapping_sub(head) {
                    Err(Error::UnexpectedEof)
                } else {
                    Ok(head.wrapping_add(offset))
                }
            }
            SeekFrom::End(offset) => {
                if offset > 0 || offset.unsigned_abs() > tail.wrapping_sub(head) {
                    Err(Error::UnexpectedEof)
                } else {
                    Ok(wrapping_add_signed(tail, offset))
                }
            }
            SeekFrom::Current(offset) => {
                let max_pos: u16 = tail.wrapping_sub(ptr);
                let max_neg: u16 = ptr.wrapping_sub(head);
                if (offset > 0 && offset.unsigned_abs() > max_pos)
                    || (offset < 0 && offset.unsigned_abs() > max_neg)
                {
                    Err(Error::UnexpectedEof)
                } else {
                    Ok(wrapping_add_signed(ptr, offset))
                }
            }
        }
    }
}

/// The `Seek` trait provides a cursor which can be moved within a stream of
/// bytes.
///
/// This is used for navigating the socket buffers, and it is designed to be
/// similar to [`std::io::Seek`].
///
/// [`std::io::Seek`]: https://doc.rust-lang.org/stable/std/io/trait.Seek.html
pub trait Seek {
    /// Seek to an offset, in bytes, within the socket buffer.
    ///
    /// Seeking beyond the limits will result [`Error::UnexpectedEof`].
    ///
    /// # Limits
    ///
    /// * [`UdpWriter`](crate::UdpWriter) is limited by socket free size.
    /// * [`UdpReader`](crate::UdpReader) is limited by the received size or
    ///   the UDP datagram length, whichever is less.
    /// * [`TcpWriter`](crate::TcpWriter) is limited by socket free size.
    /// * [`TcpReader`](crate::TcpReader) is limited by the received size.
    fn seek<E>(&mut self, pos: SeekFrom) -> Result<(), Error<E>>;

    /// Rewind to the beginning of the stream.
    ///
    /// This is a convenience method, equivalent to `seek(SeekFrom::Start(0))`.
    fn rewind(&mut self);

    /// Return the length of the stream, in bytes.
    ///
    /// * For [`TcpWriter`](crate::TcpWriter) this returns the socket free size.
    /// * For [`TcpReader`](crate::TcpReader) this returns the received size.
    /// * For [`UdpWriter`](crate::UdpWriter) this returns the socket free size.
    /// * For [`UdpReader`](crate::UdpReader) this returns the received size or
    ///   the UDP datagram length, whichever is less.
    fn stream_len(&self) -> u16;

    /// Returns the current seek position from the start of the stream.
    fn stream_position(&self) -> u16;

    /// Remaining bytes in the socket buffer from the current seek position.
    fn remain(&self) -> u16;
}

/// Socket reader trait.
pub trait Read<E> {
    /// Read data from the UDP socket, and return the number of bytes read.
    fn read(&mut self, buf: &mut [u8]) -> Result<u16, E>;

    /// Read the exact number of bytes required to fill `buf`.
    ///
    /// This function reads as many bytes as necessary to completely fill the
    /// specified buffer `buf`.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::UnexpectedEof`]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error<E>>;

    /// Mark the data as read, removing the data from the queue.
    ///
    /// For a TCP reader this removes all data up to the current pointer
    /// position from the queue.
    ///
    /// For a UDP reader this removes the UDP datagram from the queue.
    fn done(self) -> Result<(), E>;
}

/// Socket asyncrhonous reader trait.
#[cfg(feature = "async")]
pub trait AsyncRead<E> {
    type ReadFuture<'a>: core::future::Future<Output = Result<u16, E>> + 'a
    where
        Self: 'a;

    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadFuture<'a>;

    type ReadExactFuture<'a>: core::future::Future<Output = Result<(), Error<E>>> + 'a
    where
        Self: 'a;

    fn read_exact<'a>(&'a mut self, buf: &'a mut [u8]) -> Self::ReadExactFuture<'a>;
}

/// Socket writer trait.
pub trait Write<E> {
    /// Write data to the socket buffer, and return the number of bytes written.
    fn write(&mut self, buf: &[u8]) -> Result<u16, E>;

    /// Writes all the data, returning [`Error::OutOfMemory`] if the size of
    /// `buf` exceeds the free memory available in the socket buffer.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::OutOfMemory`]
    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error<E>>;

    /// Send all data previously written with [`write`] and [`write_all`].
    ///
    /// For UDP sockets the destination is set by the last call to
    /// [`Registers::set_sn_dest`], [`Udp::udp_send_to`], or
    /// [`UdpWriter::udp_send_to`].
    ///
    /// [`Registers::set_sn_dest`]: w5500_ll::Registers::set_sn_dest
    /// [`Udp::udp_send_to`]: crate::Udp::udp_send_to
    /// [`UdpWriter::udp_send_to`]: crate::UdpWriter::udp_send_to
    /// [`write_all`]: Self::write_all
    /// [`write`]: Self::write
    fn send(self) -> Result<(), E>;
}

/// Socket asyncrhonous writer trait.
#[cfg(feature = "async")]
pub trait AsyncWrite<E> {
    type WriteFuture<'a>: core::future::Future<Output = Result<u16, E>> + 'a
    where
        Self: 'a;

    fn write<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteFuture<'a>;

    type WriteAllFuture<'a>: core::future::Future<Output = Result<(), Error<E>>> + 'a
    where
        Self: 'a;

    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> Self::WriteAllFuture<'a>;
}

#[cfg(test)]
mod tests {
    use super::{Error, SeekFrom};

    #[test]
    fn seek_from_current_pos() {
        const E: Error<()> = Error::UnexpectedEof;
        type S = SeekFrom;
        assert_eq!(S::Current(0).new_ptr::<()>(0, 0, 0), Ok(0));
        assert_eq!(S::Current(1).new_ptr::<()>(0, 0, 1), Ok(1));
        assert_eq!(S::Current(1).new_ptr::<()>(1, 0, 1), Err(E));
        assert_eq!(S::Current(1).new_ptr::<()>(1, 0, 2), Ok(2));
        assert_eq!(S::Current(1).new_ptr::<()>(1, 1, 3), Ok(2));
        assert_eq!(S::Current(4096).new_ptr::<()>(0, 0, 4096), Ok(4096));
        assert_eq!(S::Current(1).new_ptr::<()>(u16::MAX, u16::MAX, 0), Ok(0));
    }

    #[test]
    fn seek_from_current_neg() {
        const E: Error<()> = Error::UnexpectedEof;
        type S = SeekFrom;
        assert_eq!(S::Current(-1).new_ptr::<()>(0, 0, 0), Err(E));
        assert_eq!(S::Current(-1).new_ptr::<()>(1, 0, 1), Ok(0));
        assert_eq!(S::Current(-2).new_ptr::<()>(1, 0, 1), Err(E));
        assert_eq!(S::Current(-1).new_ptr::<()>(0, u16::MAX, 0), Ok(u16::MAX));
        assert_eq!(S::Current(-2).new_ptr::<()>(0, u16::MAX, 0), Err(E));
    }

    #[test]
    fn seek_from_start() {
        const E: Error<()> = Error::UnexpectedEof;
        type S = SeekFrom;
        assert_eq!(S::Start(0).new_ptr::<()>(0, 0, 0), Ok(0));
        assert_eq!(S::Start(1).new_ptr::<()>(0, 0, 1), Ok(1));
        assert_eq!(S::Start(1).new_ptr::<()>(1, 0, 1), Ok(1));
        assert_eq!(S::Start(2).new_ptr::<()>(1, 0, 1), Err(E));
        assert_eq!(S::Start(2048).new_ptr::<()>(0, 2048, 8192), Ok(4096));
        assert_eq!(S::Start(1).new_ptr::<()>(0, u16::MAX, 0), Ok(0));
    }

    #[test]
    fn seek_from_end() {
        const E: Error<()> = Error::UnexpectedEof;
        type S = SeekFrom;
        assert_eq!(S::End(0).new_ptr::<()>(0, 0, 0), Ok(0));
        assert_eq!(S::End(-1).new_ptr::<()>(0, 0, 1), Ok(0));
        assert_eq!(S::End(-1).new_ptr::<()>(1, 0, 1), Ok(0));
        assert_eq!(S::End(-2).new_ptr::<()>(1, 0, 1), Err(E));
        assert_eq!(S::End(-2048).new_ptr::<()>(0, 2048, 8192), Ok(6144));
        assert_eq!(S::End(-1).new_ptr::<()>(0, u16::MAX, 0), Ok(u16::MAX));
    }
}
