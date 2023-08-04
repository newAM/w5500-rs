use crate::{
    io::{Read, Seek, SeekFrom, Write},
    port_is_unique, Error, TcpReader,
};
use core::cmp::min;
use w5500_ll::{
    net::{Ipv4Addr, SocketAddrV4},
    Protocol, Registers, Sn, SocketCommand, SocketMode, SocketStatus, TxPtrs,
};

/// W5500 UDP Header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UdpHeader {
    /// Origin IP address and port.
    pub origin: SocketAddrV4,
    /// Length of the UDP packet in bytes.
    ///
    /// This may not be equal to the length of the data in the socket buffer if
    /// the UDP packet was truncated.
    pub len: u16,
}

impl UdpHeader {
    // note: not your standard UDP datagram header
    // For a UDP socket the W5500 UDP header contains:
    // * 4 bytes origin IP
    // * 2 bytes origin port
    // * 2 bytes size
    const LEN: u16 = 8;
    const LEN_USIZE: usize = Self::LEN as usize;

    /// Deserialize a UDP header.
    fn deser(buf: [u8; Self::LEN_USIZE]) -> UdpHeader {
        UdpHeader {
            origin: SocketAddrV4::new(
                Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3]),
                u16::from_be_bytes([buf[4], buf[5]]),
            ),
            len: u16::from_be_bytes([buf[6], buf[7]]),
        }
    }
}

/// Streaming reader for a UDP socket buffer.
///
/// This implements the [`Read`] and [`Seek`] traits.
///
/// Created with [`Udp::udp_reader`].
///
/// # Example
///
/// ```no_run
/// # use ehm::eh1 as h;
/// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(h::spi::Mock::new(&[]));
/// use w5500_hl::{
///     io::Read,
///     ll::{Registers, Sn::Sn0},
///     net::{Ipv4Addr, SocketAddrV4},
///     Udp, UdpReader,
/// };
///
/// const DEST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 8081);
///
/// w5500.udp_bind(Sn0, 8080)?;
///
/// let mut reader: UdpReader<_> = w5500.udp_reader(Sn0)?;
///
/// let mut buf: [u8; 8] = [0; 8];
/// reader.read_exact(&mut buf)?;
///
/// let mut other_buf: [u8; 16] = [0; 16];
/// reader.read_exact(&mut buf)?;
///
/// // mark the datagram as done, removing it from the queue
/// reader.done()?;
/// # Ok::<(), w5500_hl::Error<_>>(())
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UdpReader<'w, W5500> {
    inner: TcpReader<'w, W5500>,
    header: UdpHeader,
}

impl<'w, W5500> Seek for UdpReader<'w, W5500> {
    fn seek<E>(&mut self, pos: SeekFrom) -> Result<(), Error<E>> {
        self.inner.seek(pos)
    }

    fn rewind(&mut self) {
        self.inner.rewind()
    }

    fn stream_len(&self) -> u16 {
        self.inner.stream_len()
    }

    fn stream_position(&self) -> u16 {
        self.inner.stream_position()
    }

    fn remain(&self) -> u16 {
        self.inner.remain()
    }
}

impl<'w, W5500: Registers> Read<W5500::Error> for UdpReader<'w, W5500> {
    fn read(&mut self, buf: &mut [u8]) -> Result<u16, W5500::Error> {
        self.inner.read(buf)
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error<W5500::Error>> {
        self.inner.read_exact(buf)
    }

    fn done(self) -> Result<(), W5500::Error> {
        self.inner
            .w5500
            .set_sn_rx_rd(self.inner.sn, self.inner.tail_ptr)?;
        self.inner
            .w5500
            .set_sn_cr(self.inner.sn, SocketCommand::Recv)?;
        Ok(())
    }
}

/// Streaming writer for a UDP socket buffer.
///
/// This implements the [`Seek`] traits.
///
/// Created with [`Udp::udp_writer`].
///
/// # Example
///
/// ```no_run
/// # use ehm::eh1 as h;
/// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(h::spi::Mock::new(&[]));
/// use w5500_hl::{
///     io::Write,
///     ll::{Registers, Sn::Sn0},
///     net::{Ipv4Addr, SocketAddrV4},
///     Udp, UdpWriter,
/// };
///
/// const DEST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 8081);
///
/// w5500.udp_bind(Sn0, 8080)?;
///
/// let mut udp_writer: UdpWriter<_> = w5500.udp_writer(Sn0)?;
///
/// let data_header: [u8; 10] = [0; 10];
/// let n_written: u16 = udp_writer.write(&data_header)?;
/// assert_eq!(usize::from(n_written), data_header.len());
///
/// let data: [u8; 123] = [0; 123];
/// let n_written: u16 = udp_writer.write(&data)?;
/// assert_eq!(usize::from(n_written), data.len());
///
/// udp_writer.udp_send_to(&DEST)?;
/// # Ok::<(), embedded_hal::spi::ErrorKind>(())
/// ```
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct UdpWriter<'w, W5500> {
    pub(crate) w5500: &'w mut W5500,
    pub(crate) sn: Sn,
    pub(crate) head_ptr: u16,
    pub(crate) tail_ptr: u16,
    pub(crate) ptr: u16,
}

impl<'w, W5500> Seek for UdpWriter<'w, W5500> {
    fn seek<E>(&mut self, pos: SeekFrom) -> Result<(), Error<E>> {
        self.ptr = pos.new_ptr(self.ptr, self.head_ptr, self.tail_ptr)?;
        Ok(())
    }

    fn rewind(&mut self) {
        self.ptr = self.head_ptr
    }

    fn stream_len(&self) -> u16 {
        self.tail_ptr.wrapping_sub(self.head_ptr)
    }

    fn stream_position(&self) -> u16 {
        self.ptr.wrapping_sub(self.head_ptr)
    }

    fn remain(&self) -> u16 {
        self.tail_ptr.wrapping_sub(self.ptr)
    }
}

impl<'w, W5500: Registers> Write<W5500::Error> for UdpWriter<'w, W5500> {
    fn write(&mut self, buf: &[u8]) -> Result<u16, W5500::Error> {
        let write_size: u16 = min(self.remain(), buf.len().try_into().unwrap_or(u16::MAX));
        if write_size != 0 {
            self.w5500
                .set_sn_tx_buf(self.sn, self.ptr, &buf[..usize::from(write_size)])?;
            self.ptr = self.ptr.wrapping_add(write_size);

            Ok(write_size)
        } else {
            Ok(0)
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Error<W5500::Error>> {
        let buf_len: u16 = buf.len().try_into().unwrap_or(u16::MAX);
        let write_size: u16 = min(self.remain(), buf_len);
        if write_size != buf_len {
            Err(Error::OutOfMemory)
        } else {
            self.w5500.set_sn_tx_buf(self.sn, self.ptr, buf)?;
            self.ptr = self.ptr.wrapping_add(write_size);
            Ok(())
        }
    }

    fn send(self) -> Result<(), W5500::Error> {
        self.w5500.set_sn_tx_wr(self.sn, self.ptr)?;
        self.w5500.set_sn_cr(self.sn, SocketCommand::Send)?;
        Ok(())
    }
}

impl<'w, W5500: Registers> UdpWriter<'w, W5500> {
    /// Send all data previously written with [`UdpWriter::write`] and
    /// [`UdpWriter::write_all`] to the given address.
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be opened as a UDP socket.
    pub fn udp_send_to(self, addr: &SocketAddrV4) -> Result<(), W5500::Error> {
        debug_assert_eq!(self.w5500.sn_sr(self.sn)?, Ok(SocketStatus::Udp));
        self.w5500.set_sn_dest(self.sn, addr)?;
        self.send()
    }
}

impl<'a, W: Registers> UdpReader<'a, W> {
    /// Get the UDP header.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::{
    ///     ll::{Registers, Sn::Sn0},
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Udp, UdpHeader, UdpReader,
    /// };
    ///
    /// const DEST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 8081);
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    ///
    /// let reader: UdpReader<_> = w5500.udp_reader(Sn0)?;
    /// let header: &UdpHeader = reader.header();
    /// # Ok::<(), w5500_hl::Error<_>>(())
    /// ```
    #[inline]
    pub fn header(&self) -> &UdpHeader {
        &self.header
    }
}

/// A W5500 UDP socket trait.
///
/// After creating a `UdpSocket` by [`bind`]ing it to a socket address,
/// data can be [sent to] and [received from] any other socket address.
///
/// As stated in the User Datagram Protocol's specification in [IETF RFC 768],
/// UDP is an unordered, unreliable protocol; refer to [`Tcp`] for the TCP trait.
///
/// # Comparison to [`std::net::UdpSocket`]
///
/// * Everything is non-blocking.
/// * There is no socket struct, you must pass a socket number as the first
///   argument to the methods.  This was simply the cleanest solution to the
///   ownership problem after some experimentation; though it certainly is not
///   the safest.
///
/// [`bind`]: Udp::udp_bind
/// [IETF RFC 768]: https://tools.ietf.org/html/rfc768
/// [received from]: Udp::udp_recv_from
/// [sent to]: Udp::udp_send_to
/// [`Tcp`]: crate::Tcp
/// [`std::net::UdpSocket`]: https://doc.rust-lang.org/std/net/struct.UdpSocket.html
pub trait Udp: Registers {
    /// Binds the socket to the given port.
    ///
    /// This will close the socket, which will reset the RX and TX buffers.
    ///
    /// # Comparison to [`std::net::UdpSocket::bind`]
    ///
    /// This method accepts a port instead of a [`net::SocketAddrV4`], this is
    /// because the IP address is global for the device, set by the
    /// [source IP register], and cannot be set on a per-socket basis.
    ///
    /// Additionally you can only provide one port, instead of iterable
    /// addresses to bind.
    ///
    /// # Panics
    ///
    /// * (debug) The port must not be in use by any other socket on the W5500.
    ///
    /// # Example
    ///
    /// Bind the first socket to port 8080.
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::ll::{Registers, Sn::Sn0};
    /// use w5500_hl::Udp;
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    ///
    /// [`net::SocketAddrV4`]: [crate::net::SocketAddrV4]
    /// [`std::net::UdpSocket::bind`]: https://doc.rust-lang.org/std/net/struct.UdpSocket.html#method.bind
    /// [source IP register]: w5500_ll::Registers::sipr
    fn udp_bind(&mut self, sn: Sn, port: u16) -> Result<(), Self::Error> {
        debug_assert!(
            port_is_unique(self, sn, port)?,
            "Local port {port} is in use"
        );

        self.set_sn_cr(sn, SocketCommand::Close)?;
        // This will not hang, the socket status will always change to closed
        // after a close command.
        // (unless you do somthing silly like holding the W5500 in reset)
        while self.sn_sr(sn)? != Ok(SocketStatus::Closed) {}
        self.set_sn_port(sn, port)?;
        const MODE: SocketMode = SocketMode::DEFAULT.set_protocol(Protocol::Udp);
        self.set_sn_mr(sn, MODE)?;
        self.set_sn_cr(sn, SocketCommand::Open)?;
        // This will not hang, the socket status will always change to Udp
        // after a open command with SN_MR set to UDP.
        // (unless you do somthing silly like holding the W5500 in reset)
        while self.sn_sr(sn)? != Ok(SocketStatus::Udp) {}
        Ok(())
    }

    /// Receives a single datagram message on the socket.
    /// On success, returns the number of bytes read and the origin.
    ///
    /// The function must be called with valid byte array `buf` of sufficient
    /// size to hold the message bytes.
    /// If a message is too long to fit in the supplied buffer, excess bytes
    /// will be discarded.
    ///
    /// # Comparison to [`std::net::UdpSocket::recv_from`]
    ///
    /// * This method will always discard excess bytes from the socket buffer.
    /// * This method is non-blocking, use [`block`] to treat it as blocking.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::WouldBlock`]
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be opened as a UDP socket.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::{
    ///     block,
    ///     ll::{Registers, Sn::Sn0},
    ///     Udp,
    /// };
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    /// let mut buf = [0; 10];
    /// let (number_of_bytes, src_addr) = block!(w5500.udp_recv_from(Sn0, &mut buf))?;
    ///
    /// // panics if bytes were discarded
    /// assert!(
    ///     usize::from(number_of_bytes) < buf.len(),
    ///     "Buffer was too small to receive all data"
    /// );
    ///
    /// let filled_buf = &mut buf[..number_of_bytes.into()];
    /// # Ok::<(), w5500_hl::Error<_>>(())
    /// ```
    ///
    /// [`std::net::UdpSocket::recv_from`]: https://doc.rust-lang.org/std/net/struct.UdpSocket.html#method.recv_from
    /// [`block`]: crate::block
    fn udp_recv_from(
        &mut self,
        sn: Sn,
        buf: &mut [u8],
    ) -> Result<(u16, SocketAddrV4), Error<Self::Error>> {
        let rsr: u16 = match self.sn_rx_rsr(sn)?.checked_sub(UdpHeader::LEN) {
            Some(rsr) => rsr,
            // nothing to recieve
            None => return Err(Error::WouldBlock),
        };

        debug_assert_eq!(self.sn_sr(sn)?, Ok(SocketStatus::Udp));

        let mut ptr: u16 = self.sn_rx_rd(sn)?;
        let mut header: [u8; UdpHeader::LEN_USIZE] = [0; UdpHeader::LEN_USIZE];
        self.sn_rx_buf(sn, ptr, &mut header)?;
        ptr = ptr.wrapping_add(UdpHeader::LEN);
        let header: UdpHeader = UdpHeader::deser(header);

        // not all data as indicated by the header has been buffered
        if rsr < header.len {
            return Err(Error::WouldBlock);
        }

        let read_size: u16 = min(header.len, buf.len().try_into().unwrap_or(u16::MAX));
        if read_size != 0 {
            self.sn_rx_buf(sn, ptr, &mut buf[..read_size.into()])?;
        }
        ptr = ptr.wrapping_add(header.len);
        self.set_sn_rx_rd(sn, ptr)?;
        self.set_sn_cr(sn, SocketCommand::Recv)?;
        Ok((read_size, header.origin))
    }

    /// Receives a single datagram message on the socket, without removing it
    /// from the queue.
    /// On success, returns the number of bytes read and the UDP header.
    ///
    /// # Comparison to [`std::net::UdpSocket::peek_from`]
    ///
    /// * This method will never discard excess bytes from the socket buffer.
    /// * This method is non-blocking, use [`block`] to treat it as blocking.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::WouldBlock`]
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be opened as a UDP socket.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::{
    ///     block,
    ///     ll::{Registers, Sn::Sn0},
    ///     Udp,
    /// };
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    /// let mut buf = [0; 10];
    /// let (number_of_bytes, udp_header) = block!(w5500.udp_peek_from(Sn0, &mut buf))?;
    ///
    /// // panics if buffer was too small
    /// assert!(
    ///     usize::from(number_of_bytes) > buf.len(),
    ///     "Buffer was of len {} too small to receive all data: {} / {} bytes read",
    ///     buf.len(),
    ///     number_of_bytes,
    ///     udp_header.len
    /// );
    ///
    /// let filled_buf = &mut buf[..number_of_bytes.into()];
    /// # Ok::<(), w5500_hl::Error<_>>(())
    /// ```
    ///
    /// [`std::net::UdpSocket::peek_from`]: https://doc.rust-lang.org/std/net/struct.UdpSocket.html#method.peek_from
    /// [`block`]: crate::block
    fn udp_peek_from(
        &mut self,
        sn: Sn,
        buf: &mut [u8],
    ) -> Result<(u16, UdpHeader), Error<Self::Error>> {
        let rsr: u16 = match self.sn_rx_rsr(sn)?.checked_sub(UdpHeader::LEN) {
            Some(rsr) => rsr,
            // nothing to recieve
            None => return Err(Error::WouldBlock),
        };

        debug_assert_eq!(self.sn_sr(sn)?, Ok(SocketStatus::Udp));

        let mut ptr: u16 = self.sn_rx_rd(sn)?;
        let mut header: [u8; UdpHeader::LEN_USIZE] = [0; UdpHeader::LEN_USIZE];
        self.sn_rx_buf(sn, ptr, &mut header)?;
        ptr = ptr.wrapping_add(UdpHeader::LEN);
        let header: UdpHeader = UdpHeader::deser(header);

        // not all data as indicated by the header has been buffered
        if rsr < header.len {
            return Err(Error::WouldBlock);
        }

        let read_size: u16 = min(header.len, buf.len().try_into().unwrap_or(u16::MAX));
        if read_size != 0 {
            self.sn_rx_buf(sn, ptr, &mut buf[..read_size.into()])?;
        }

        Ok((read_size, header))
    }

    /// Receives the origin and size of the next datagram available on the
    /// socket, without removing it from the queue.
    ///
    /// There is no [`std::net`](https://doc.rust-lang.org/std/net) equivalent
    /// for this method.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::WouldBlock`]
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be opened as a UDP socket.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::{
    ///     block,
    ///     ll::{Registers, Sn::Sn0},
    ///     Udp, UdpHeader,
    /// };
    /// // global_allocator is currently available on nightly for embedded rust
    /// extern crate alloc;
    /// use alloc::vec::{self, Vec};
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    /// let udp_header: UdpHeader = block!(w5500.udp_peek_from_header(Sn0))?;
    ///
    /// let mut buf: Vec<u8> = vec![0; udp_header.len.into()];
    /// let (number_of_bytes, source) = block!(w5500.udp_recv_from(Sn0, &mut buf))?;
    /// // this can assert if the UDP datagram was truncated
    /// // e.g. due to an insufficient socket buffer size
    /// assert_eq!(udp_header.len, number_of_bytes);
    /// # Ok::<(), w5500_hl::Error<_>>(())
    /// ```
    fn udp_peek_from_header(&mut self, sn: Sn) -> Result<UdpHeader, Error<Self::Error>> {
        let rsr: u16 = self.sn_rx_rsr(sn)?;

        // nothing to recieve
        if rsr < UdpHeader::LEN {
            return Err(Error::WouldBlock);
        }

        debug_assert_eq!(self.sn_sr(sn)?, Ok(SocketStatus::Udp));

        let ptr: u16 = self.sn_rx_rd(sn)?;
        let mut header: [u8; UdpHeader::LEN_USIZE] = [0; UdpHeader::LEN_USIZE];
        self.sn_rx_buf(sn, ptr, &mut header)?;
        Ok(UdpHeader::deser(header))
    }

    /// Sends data on the socket to the given address.
    /// On success, returns the number of bytes written.
    ///
    /// # Comparison to [`std::net::UdpSocket::send_to`]
    ///
    /// * You cannot transmit more than `u16::MAX` bytes at once.
    /// * You can only provide one destination.
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be opened as a UDP socket.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::{
    ///     ll::{Registers, Sn::Sn0},
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Udp,
    /// };
    ///
    /// const DEST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 8081);
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    /// let buf: [u8; 10] = [0; 10];
    /// let tx_bytes: u16 = w5500.udp_send_to(Sn0, &buf, &DEST)?;
    /// assert_eq!(usize::from(tx_bytes), buf.len());
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    ///
    /// [`std::net::UdpSocket::send_to`]: https://doc.rust-lang.org/std/net/struct.UdpSocket.html#method.send_to
    fn udp_send_to(&mut self, sn: Sn, buf: &[u8], addr: &SocketAddrV4) -> Result<u16, Self::Error> {
        self.set_sn_dest(sn, addr)?;
        self.udp_send(sn, buf)
    }

    /// Sends data on the socket to the given address.
    /// On success, returns the number of bytes written.
    ///
    /// This will transmit only if there is enough free space in the W5500
    /// transmit buffer.
    ///
    /// # Comparison to [`std::net::UdpSocket::send_to`]
    ///
    /// * You cannot transmit more than `u16::MAX` bytes at once.
    /// * You can only provide one destination.
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be opened as a UDP socket.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::{
    ///     ll::{Registers, Sn::Sn0},
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Udp,
    /// };
    ///
    /// const DEST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 8081);
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    /// let buf: [u8; 10] = [0; 10];
    /// let tx_bytes: u16 = w5500.udp_send_to_if_free(Sn0, &buf, &DEST)?;
    /// assert_eq!(usize::from(tx_bytes), buf.len());
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    ///
    /// [`std::net::UdpSocket::send_to`]: https://doc.rust-lang.org/std/net/struct.UdpSocket.html#method.send_to
    fn udp_send_to_if_free(
        &mut self,
        sn: Sn,
        buf: &[u8],
        addr: &SocketAddrV4,
    ) -> Result<u16, Self::Error> {
        self.set_sn_dest(sn, addr)?;
        self.udp_send_if_free(sn, buf)
    }

    /// Sends data to the currently configured destination.
    /// On success, returns the number of bytes written.
    ///
    /// The destination is set by the last call to [`Registers::set_sn_dest`],
    /// [`Udp::udp_send_to`], or [`UdpWriter::udp_send_to`].
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be opened as a UDP socket.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::{
    ///     ll::{Registers, Sn::Sn0},
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Udp,
    /// };
    ///
    /// const DEST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 8081);
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    /// let buf: [u8; 10] = [0; 10];
    /// let tx_bytes: u16 = w5500.udp_send_to(Sn0, &buf, &DEST)?;
    /// assert_eq!(usize::from(tx_bytes), buf.len());
    /// // send the same to the same destination
    /// let tx_bytes: u16 = w5500.udp_send(Sn0, &buf)?;
    /// assert_eq!(usize::from(tx_bytes), buf.len());
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    ///
    /// [`UdpWriter::udp_send_to`]: crate::UdpWriter::udp_send_to
    fn udp_send(&mut self, sn: Sn, buf: &[u8]) -> Result<u16, Self::Error> {
        debug_assert_eq!(self.sn_sr(sn)?, Ok(SocketStatus::Udp));

        let data_len: u16 = u16::try_from(buf.len()).unwrap_or(u16::MAX);
        let free_size: u16 = self.sn_tx_fsr(sn)?;
        let tx_bytes: u16 = min(data_len, free_size);
        if tx_bytes != 0 {
            let ptr: u16 = self.sn_tx_wr(sn)?;
            self.set_sn_tx_buf(sn, ptr, &buf[..tx_bytes.into()])?;
            self.set_sn_tx_wr(sn, ptr.wrapping_add(tx_bytes))?;
            self.set_sn_cr(sn, SocketCommand::Send)?;
        }
        Ok(tx_bytes)
    }

    /// Sends data to the currently configured destination.
    /// On success, returns the number of bytes written.
    ///
    /// The destination is set by the last call to [`Registers::set_sn_dest`],
    /// [`Udp::udp_send_to`], or [`UdpWriter::udp_send_to`].
    ///
    /// This will transmit only if there is enough free space in the W5500
    /// transmit buffer.
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be opened as a UDP socket.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # let mut w5500 = w5500_ll::eh1::vdm::W5500::new(ehm::eh1::spi::Mock::new(&[]));
    /// use w5500_hl::{
    ///     ll::{Registers, Sn::Sn0},
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Udp,
    /// };
    ///
    /// const DEST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 8081);
    ///
    /// w5500.udp_bind(Sn0, 8080)?;
    /// let buf: [u8; 10] = [0; 10];
    /// let tx_bytes: u16 = w5500.udp_send_to_if_free(Sn0, &buf, &DEST)?;
    /// assert_eq!(usize::from(tx_bytes), buf.len());
    /// // send the same to the same destination
    /// let tx_bytes: u16 = w5500.udp_send_if_free(Sn0, &buf)?;
    /// assert_eq!(usize::from(tx_bytes), buf.len());
    /// # Ok::<(), embedded_hal::spi::ErrorKind>(())
    /// ```
    ///
    /// [`UdpWriter::udp_send_to`]: crate::UdpWriter::udp_send_to
    fn udp_send_if_free(&mut self, sn: Sn, buf: &[u8]) -> Result<u16, Self::Error> {
        debug_assert_eq!(self.sn_sr(sn)?, Ok(SocketStatus::Udp));

        let data_len: u16 = match u16::try_from(buf.len()) {
            Ok(l) => l,
            Err(_) => return Ok(0),
        };
        let free_size: u16 = self.sn_tx_fsr(sn)?;
        if data_len <= free_size {
            let ptr: u16 = self.sn_tx_wr(sn)?;
            self.set_sn_tx_buf(sn, ptr, buf)?;
            self.set_sn_tx_wr(sn, ptr.wrapping_add(data_len))?;
            self.set_sn_cr(sn, SocketCommand::Send)?;
        }
        Ok(data_len)
    }

    /// Create a UDP reader.
    ///
    /// This returns a [`UdpReader`] structure, which contains functions to
    /// stream data from the W5500 socket buffers incrementally.
    ///
    /// This will return [`Error::WouldBlock`] if there is no data to read.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::WouldBlock`]
    ///
    /// # Example
    ///
    /// See [`UdpReader`].
    fn udp_reader(&mut self, sn: Sn) -> Result<UdpReader<Self>, Error<Self::Error>>
    where
        Self: Sized,
    {
        debug_assert_eq!(self.sn_sr(sn)?, Ok(SocketStatus::Udp));

        let rsr: u16 = match self.sn_rx_rsr(sn)?.checked_sub(UdpHeader::LEN) {
            Some(rsr) => rsr,
            // nothing to recieve
            None => return Err(Error::WouldBlock),
        };

        let sn_rx_rd: u16 = self.sn_rx_rd(sn)?;
        let mut header: [u8; UdpHeader::LEN_USIZE] = [0; UdpHeader::LEN_USIZE];
        self.sn_rx_buf(sn, sn_rx_rd, &mut header)?;
        let header: UdpHeader = UdpHeader::deser(header);

        // limit to the length of the first datagram if we have more than a
        // single datagram enqueued
        let rsr_or_datagram_len: u16 = min(header.len, rsr);

        let head_ptr: u16 = sn_rx_rd.wrapping_add(UdpHeader::LEN);

        Ok(UdpReader {
            inner: TcpReader {
                w5500: self,
                sn,
                head_ptr,
                tail_ptr: head_ptr.wrapping_add(rsr_or_datagram_len),
                ptr: head_ptr,
            },
            header,
        })
    }

    /// Create a UDP writer.
    ///
    /// This returns a [`UdpWriter`] structure, which contains functions to
    /// stream data to the W5500 socket buffers incrementally.
    ///
    /// # Example
    ///
    /// See [`UdpWriter`].
    fn udp_writer(&mut self, sn: Sn) -> Result<UdpWriter<Self>, Self::Error>
    where
        Self: Sized,
    {
        let tx_ptrs: TxPtrs = self.sn_tx_ptrs(sn)?;

        Ok(UdpWriter {
            w5500: self,
            sn,
            head_ptr: tx_ptrs.wr,
            tail_ptr: tx_ptrs.wr.wrapping_add(tx_ptrs.fsr),
            ptr: tx_ptrs.wr,
        })
    }
}

/// Implement the UDP trait for any structure that implements [`w5500_ll::Registers`].
impl<T> Udp for T where T: Registers {}
