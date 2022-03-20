#[cfg(feature = "defmt")]
use dfmt as defmt;

use crate::{port_is_unique, Error, Read, Seek, SeekFrom};
use core::cmp::min;
use w5500_ll::{
    net::SocketAddrV4, Protocol, Registers, Sn, SocketCommand, SocketMode, SocketStatus,
};

#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TcpReader<'a, W: Registers> {
    pub(crate) w5500: &'a mut W,
    pub(crate) sn: Sn,
    pub(crate) head_ptr: u16,
    pub(crate) tail_ptr: u16,
    pub(crate) ptr: u16,
}

impl<'a, W: Registers> Seek for TcpReader<'a, W> {
    fn seek(&mut self, pos: SeekFrom) {
        self.ptr = pos.new_ptr(self.ptr, self.head_ptr, self.tail_ptr);
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

impl<'a, W: Registers> Read<'a, W> for TcpReader<'a, W> {
    fn read(&mut self, buf: &mut [u8]) -> Result<u16, W::Error> {
        let read_size: u16 = min(self.remain(), buf.len().try_into().unwrap_or(u16::MAX));
        if read_size != 0 {
            self.w5500
                .sn_rx_buf(self.sn, self.ptr, &mut buf[..usize::from(read_size)])?;
            self.ptr = self.ptr.wrapping_add(read_size);

            Ok(read_size)
        } else {
            Ok(0)
        }
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), Error<W::Error>> {
        let buf_len: u16 = buf.len().try_into().unwrap_or(u16::MAX);
        let read_size: u16 = min(self.remain(), buf_len);
        if read_size != buf_len {
            Err(Error::UnexpectedEof)
        } else {
            self.w5500.sn_rx_buf(self.sn, self.ptr, buf)?;
            self.ptr = self.ptr.wrapping_add(read_size);
            Ok(())
        }
    }

    fn done(self) -> Result<&'a mut W, W::Error> {
        self.w5500.set_sn_rx_rd(self.sn, self.tail_ptr)?;
        self.w5500.set_sn_cr(self.sn, SocketCommand::Recv)?;
        Ok(self.w5500)
    }

    #[inline]
    fn ignore(self) -> &'a mut W {
        self.w5500
    }
}

/// A W5500 TCP trait.
pub trait Tcp: Registers {
    /// Starts the 3-way TCP handshake with the remote host.
    ///
    /// This method is used to create and interact with a TCP stream between
    /// a local host and a remote socket.
    ///
    /// After initiating a connection with [`tcp_connect`] and recieving the
    /// [`con`] interrupt data can be transmitting by using [`tcp_read`] and
    /// [`tcp_write`].
    ///
    /// Calling this method **does not** mean the socket will be connected
    /// afterwards, this simply starts the three way handshake.
    ///
    /// After calling this method you will eventually get one of 3 interrupts on
    /// the socket:
    ///
    /// 1. [`con`](w5500_ll::SocketInterrupt::con_raised)
    /// 2. [`discon`](w5500_ll::SocketInterrupt::discon_raised)
    /// 3. [`timeout`](w5500_ll::SocketInterrupt::timeout_raised)
    ///
    /// # Arguments
    ///
    /// * `socket` - The socket number to use for this TCP stream.
    /// * `port` - The local port to use for the TCP connection.
    /// * `addr` - Address of the remote host to connect to.
    ///
    /// # Panics
    ///
    /// * (debug) The port must not be in use by any other socket on the W5500.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use embedded_hal_mock as h;
    /// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
    /// use w5500_hl::{
    ///     ll::{Registers, Sn, SocketInterrupt},
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Tcp,
    /// };
    ///
    /// const MQTT_SOCKET: Sn = Sn::Sn0;
    /// const MQTT_SOURCE_PORT: u16 = 33650;
    /// const MQTT_SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 2, 10), 1883);
    ///
    /// w5500.tcp_connect(MQTT_SOCKET, MQTT_SOURCE_PORT, &MQTT_SERVER)?;
    ///
    /// // wait for a socket interrupt
    /// // you should use the actual interrupt pin, polling is just for demonstration
    /// loop {
    ///     let sn_ir: SocketInterrupt = w5500.sn_ir(MQTT_SOCKET)?;
    ///
    ///     // in reality you will want to handle disconnections gracefully with retries
    ///     assert!(!sn_ir.discon_raised());
    ///     assert!(!sn_ir.timeout_raised());
    ///
    ///     // connection succeded
    ///     if sn_ir.con_raised() {
    ///         break;
    ///     }
    /// }
    /// # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
    /// ```
    ///
    /// [`tcp_write`]: Tcp::tcp_write
    /// [`tcp_read`]: Tcp::tcp_read
    /// [`tcp_connect`]: Tcp::tcp_connect
    /// [`con`]: w5500_ll::SocketInterrupt::con_raised
    fn tcp_connect(&mut self, sn: Sn, port: u16, addr: &SocketAddrV4) -> Result<(), Self::Error> {
        debug_assert!(
            port_is_unique(self, sn, port)?,
            "Local port {port} is in use"
        );

        self.set_sn_cr(sn, SocketCommand::Close)?;
        // This will not hang, the socket status will always change to closed
        // after a close command.
        // (unless you do somthing silly like holding the W5500 in reset)
        loop {
            if self.sn_sr(sn)? == Ok(SocketStatus::Closed) {
                break;
            }
        }
        const MODE: SocketMode = SocketMode::DEFAULT.set_protocol(Protocol::Tcp);
        self.set_sn_mr(sn, MODE)?;
        self.set_sn_port(sn, port)?;
        self.set_sn_cr(sn, SocketCommand::Open)?;
        self.set_sn_dest(sn, addr)?;
        // This will not hang, the socket status will always change to Init
        // after a open command with SN_MR set to TCP.
        // (unless you do somthing silly like holding the W5500 in reset)
        loop {
            if self.sn_sr(sn)? == Ok(SocketStatus::Init) {
                break;
            }
        }
        self.set_sn_cr(sn, SocketCommand::Connect)
    }

    /// Open a TCP listener on the given port.
    ///
    /// After opening a listener with [`tcp_listen`] and recieving the
    /// [`con`] interrupt data can be transmitting by using [`tcp_read`] and
    /// [`tcp_write`].
    ///
    /// # Arguments
    ///
    /// * `socket` - The socket number to use for this TCP listener.
    /// * `port` - The local port to listen for remote connections on.
    ///
    /// # Panics
    ///
    /// * (debug) The port must not be in use by any other socket on the W5500.
    ///
    /// # Example
    ///
    /// Create an HTTP server.
    ///
    /// ```no_run
    /// # use embedded_hal_mock as h;
    /// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
    /// use w5500_hl::{
    ///     ll::{Registers, Sn, SocketInterrupt},
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Tcp,
    /// };
    /// // global_allocator is currently avaliable on nightly for embedded rust
    /// extern crate alloc;
    /// use alloc::vec::{self, Vec};
    ///
    /// const HTTP_SOCKET: Sn = Sn::Sn1;
    /// const HTTP_PORT: u16 = 80;
    ///
    /// // start serving
    /// w5500.tcp_listen(HTTP_SOCKET, HTTP_PORT)?;
    ///
    /// // wait for the RECV interrupt, indicating there is data to read from a client
    /// loop {
    ///     let sn_ir = w5500.sn_ir(HTTP_SOCKET).unwrap();
    ///     if sn_ir.recv_raised() {
    ///         w5500.set_sn_ir(HTTP_SOCKET, sn_ir).unwrap();
    ///         break;
    ///     }
    ///     if sn_ir.discon_raised() | sn_ir.timeout_raised() {
    ///         panic!("Socket disconnected while waiting for RECV");
    ///     }
    /// }
    ///
    /// let mut buf: Vec<u8> = vec![0; 256];
    /// let rx_bytes: u16 = w5500.tcp_read(HTTP_SOCKET, &mut buf)?;
    /// // Truncate the buffer to the number of bytes read
    /// // Safety: BUF is only borrowed mutably in one location
    /// let filled_buf: &[u8] = &buf[..rx_bytes.into()];
    ///
    /// // parse HTTP request here using filled_buf
    /// # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
    /// ```
    ///
    /// [`tcp_write`]: Tcp::tcp_write
    /// [`tcp_read`]: Tcp::tcp_read
    /// [`tcp_listen`]: Tcp::tcp_listen
    /// [`con`]: w5500_ll::SocketInterrupt::con_raised
    fn tcp_listen(&mut self, sn: Sn, port: u16) -> Result<(), Self::Error> {
        debug_assert!(
            port_is_unique(self, sn, port)?,
            "Local port {port} is in use"
        );

        self.set_sn_cr(sn, SocketCommand::Close)?;
        // This will not hang, the socket status will always change to closed
        // after a close command.
        // (unless you do somthing silly like holding the W5500 in reset)
        loop {
            if self.sn_sr(sn)? == Ok(SocketStatus::Closed) {
                break;
            }
        }
        const MODE: SocketMode = SocketMode::DEFAULT.set_protocol(Protocol::Tcp);
        self.set_sn_mr(sn, MODE)?;
        self.set_sn_port(sn, port)?;
        self.set_sn_cr(sn, SocketCommand::Open)?;
        // This will not hang, the socket status will always change to Init
        // after a open command with SN_MR set to TCP.
        // (unless you do somthing silly like holding the W5500 in reset)
        loop {
            if self.sn_sr(sn)? == Ok(SocketStatus::Init) {
                break;
            }
        }
        self.set_sn_cr(sn, SocketCommand::Listen)
    }

    /// Read data from the remote host, returning the number of bytes read.
    ///
    /// You should wait for the socket [`recv`] interrupt before calling this method.
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be an [`Established`] TCP socket.
    ///
    /// # Example
    ///
    /// Send a MQTT CONNECT packet and read a CONNACK.
    ///
    /// ```no_run
    /// # use embedded_hal_mock as h;
    /// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
    /// use w5500_hl::{
    ///     ll::{Registers, Sn, SocketInterrupt},
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Tcp,
    /// };
    ///
    /// const MQTT_SOCKET: Sn = Sn::Sn0;
    /// const MQTT_SOURCE_PORT: u16 = 33650;
    /// const MQTT_SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 2, 10), 1883);
    ///
    /// w5500.tcp_connect(MQTT_SOCKET, MQTT_SOURCE_PORT, &MQTT_SERVER)?;
    ///
    /// // ... wait for a CON interrupt
    ///
    /// const CONNECT: [u8; 14] = [
    ///     0x10, 0x0C, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04, 0x02, 0x0E, 0x10, 0x00, 0x00,
    /// ];
    /// let tx_bytes: u16 = w5500.tcp_write(MQTT_SOCKET, &CONNECT)?;
    /// assert_eq!(usize::from(tx_bytes), CONNECT.len());
    ///
    /// // ... wait for a RECV interrupt
    ///
    /// let mut buf = [0; 10];
    /// let rx_bytes: u16 = w5500.tcp_read(MQTT_SOCKET, &mut buf)?;
    /// let filled_buf = &buf[..rx_bytes.into()];
    /// # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
    /// ```
    ///
    /// [`Established`]: w5500_ll::SocketStatus::Established
    /// [`recv`]: w5500_ll::SocketInterrupt::recv_raised
    fn tcp_read(&mut self, sn: Sn, buf: &mut [u8]) -> Result<u16, Self::Error> {
        debug_assert!(!matches!(
            self.sn_sr(sn)?,
            Ok(SocketStatus::Udp) | Ok(SocketStatus::Init) | Ok(SocketStatus::Macraw)
        ));

        let rx_bytes: u16 = {
            let rsr: u16 = self.sn_rx_rsr(sn)?;
            min(rsr, u16::try_from(buf.len()).unwrap_or(u16::MAX))
        };
        if rx_bytes != 0 {
            let ptr: u16 = self.sn_rx_rd(sn)?;
            self.sn_rx_buf(sn, ptr, &mut buf[..usize::from(rx_bytes)])?;
            self.set_sn_rx_rd(sn, ptr.wrapping_add(rx_bytes))?;
            self.set_sn_cr(sn, SocketCommand::Recv)?;
        }
        Ok(rx_bytes)
    }

    /// Send data to the remote host, returning the number of bytes written.
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be an [`Established`] TCP socket.
    ///
    /// # Example
    ///
    /// Send a MQTT CONNECT packet.
    ///
    /// ```no_run
    /// # use embedded_hal_mock as h;
    /// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
    /// use w5500_hl::{
    ///     ll::{Registers, Sn, SocketInterrupt},
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Tcp,
    /// };
    ///
    /// const MQTT_SOCKET: Sn = Sn::Sn0;
    /// const MQTT_SOURCE_PORT: u16 = 33650;
    /// const MQTT_SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 2, 10), 1883);
    ///
    /// w5500.tcp_connect(MQTT_SOCKET, MQTT_SOURCE_PORT, &MQTT_SERVER)?;
    ///
    /// // ... wait for a CON interrupt
    ///
    /// const CONNECT: [u8; 14] = [
    ///     0x10, 0x0C, 0x00, 0x04, b'M', b'Q', b'T', b'T', 0x04, 0x02, 0x0E, 0x10, 0x00, 0x00,
    /// ];
    /// let tx_bytes: u16 = w5500.tcp_write(MQTT_SOCKET, &CONNECT)?;
    /// assert_eq!(usize::from(tx_bytes), CONNECT.len());
    /// # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
    /// ```
    ///
    /// [`Established`]: w5500_ll::SocketStatus::Established
    fn tcp_write(&mut self, sn: Sn, buf: &[u8]) -> Result<u16, Self::Error> {
        debug_assert!(!matches!(
            self.sn_sr(sn)?,
            Ok(SocketStatus::Udp) | Ok(SocketStatus::Init) | Ok(SocketStatus::Macraw)
        ));

        let tx_bytes: u16 = {
            let data_len: u16 = u16::try_from(buf.len()).unwrap_or(u16::MAX);
            let free_size: u16 = self.sn_tx_fsr(sn)?;
            min(data_len, free_size)
        };
        if tx_bytes != 0 {
            let ptr: u16 = self.sn_tx_wr(sn)?;
            self.set_sn_tx_buf(sn, ptr, &buf[..usize::from(tx_bytes)])?;
            self.set_sn_tx_wr(sn, ptr.wrapping_add(tx_bytes))?;
            self.set_sn_cr(sn, SocketCommand::Send)?;
        }
        Ok(tx_bytes)
    }

    /// Disconnect from the peer.
    ///
    /// If the disconnect is successful (FIN/ACK packet is received) the socket
    /// status changes to [`Closed`], otherwise TCP<sub>TO</sub> occurs, the
    /// [timeout interrupt] is raised, and the socket status changes to
    /// [`Closed`].
    ///
    /// # Panics
    ///
    /// * (debug) The socket must be an [`Established`] TCP socket.
    ///
    /// # Example
    ///
    /// Connect and disconnect from a MQTT server.
    ///
    /// ```no_run
    /// # use embedded_hal_mock as h;
    /// # let mut w5500 = w5500_ll::blocking::vdm::W5500::new(h::spi::Mock::new(&[]), h::pin::Mock::new(&[]));
    /// use w5500_hl::{
    ///     ll::{Registers, Sn, SocketInterrupt},
    ///     net::{Ipv4Addr, SocketAddrV4},
    ///     Tcp,
    /// };
    ///
    /// const MQTT_SOCKET: Sn = Sn::Sn0;
    /// const MQTT_SOURCE_PORT: u16 = 33650;
    /// const MQTT_SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 2, 10), 1883);
    ///
    /// w5500.tcp_connect(MQTT_SOCKET, MQTT_SOURCE_PORT, &MQTT_SERVER)?;
    ///
    /// // ... wait for a CON interrupt
    ///
    /// w5500.tcp_disconnect(MQTT_SOCKET)?;
    /// # Ok::<(), w5500_hl::ll::blocking::vdm::Error<_, _>>(())
    /// ```
    ///
    /// [`Closed`]: w5500_ll::SocketStatus::Closed
    /// [`Established`]: w5500_ll::SocketStatus::Established
    /// [timeout interrupt]: w5500_ll::SocketInterrupt::timeout_raised
    fn tcp_disconnect(&mut self, sn: Sn) -> Result<(), Self::Error> {
        debug_assert!(!matches!(
            self.sn_sr(sn)?,
            Ok(SocketStatus::Udp) | Ok(SocketStatus::Init) | Ok(SocketStatus::Macraw)
        ));
        self.set_sn_cr(sn, SocketCommand::Disconnect)
    }

    /// Create a TCP reader.
    ///
    /// This returns a [`TcpReader`] structure, which contains functions to
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
    /// See [`TcpReader`].
    fn tcp_reader(&mut self, sn: Sn) -> Result<TcpReader<Self>, Error<Self::Error>>
    where
        Self: Sized,
    {
        debug_assert!(!matches!(
            self.sn_sr(sn)?,
            Ok(SocketStatus::Udp) | Ok(SocketStatus::Init) | Ok(SocketStatus::Macraw)
        ));

        let sn_rx_rsr: u16 = self.sn_rx_rsr(sn)?;
        let sn_rx_rd: u16 = self.sn_rx_rd(sn)?;

        Ok(TcpReader {
            w5500: self,
            sn,
            head_ptr: sn_rx_rd,
            tail_ptr: sn_rx_rd.wrapping_add(sn_rx_rsr),
            ptr: sn_rx_rd,
        })
    }
}

/// Implement the TCP trait for any structure that implements [`w5500_ll::Registers`].
impl<T> Tcp for T where T: Registers {}
