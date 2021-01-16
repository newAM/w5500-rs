pub use w5500_ll::net::{Eui48Addr, Ipv4Addr};

/// An IPv4 socket address.
///
/// This is mostly ripped directly from [`std::net::SocketAddrV4`].
///
/// IPv4 socket addresses consist of an [`IPv4` address] and a 16-bit port number, as
/// stated in [IETF RFC 793].
///
/// [IETF RFC 793]: https://tools.ietf.org/html/rfc793
/// [`IPv4` address]: Ipv4Addr
/// [`std::net::SocketAddrV4`]: https://doc.rust-lang.org/std/net/struct.SocketAddrV4.html
///
/// # Examples
///
/// ```
/// use w5500_hl::net::{Ipv4Addr, SocketAddrV4};
///
/// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
///
/// assert_eq!(socket.ip(), &Ipv4Addr::new(127, 0, 0, 1));
/// assert_eq!(socket.port(), 8080);
/// ```
pub struct SocketAddrV4 {
    ip: Ipv4Addr,
    port: u16,
}

impl SocketAddrV4 {
    /// Creates a new socket address from an [`IPv4` address] and a port number.
    ///
    /// [`IPv4` address]: Ipv4Addr
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_hl::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// ```
    pub const fn new(ip: Ipv4Addr, port: u16) -> SocketAddrV4 {
        SocketAddrV4 { ip, port }
    }

    /// Returns the IP address associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_hl::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// assert_eq!(socket.ip(), &Ipv4Addr::new(127, 0, 0, 1));
    /// ```
    pub const fn ip(&self) -> &Ipv4Addr {
        &self.ip
    }

    /// Changes the IP address associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_hl::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let mut socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// socket.set_ip(Ipv4Addr::new(192, 168, 0, 1));
    /// assert_eq!(socket.ip(), &Ipv4Addr::new(192, 168, 0, 1));
    /// ```
    pub fn set_ip(&mut self, new_ip: Ipv4Addr) {
        self.ip = new_ip
    }

    /// Returns the port number associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_hl::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// assert_eq!(socket.port(), 8080);
    /// ```
    pub const fn port(&self) -> u16 {
        self.port
    }

    /// Changes the port number associated with this socket address.
    ///
    /// # Examples
    ///
    /// ```
    /// use w5500_hl::net::{SocketAddrV4, Ipv4Addr};
    ///
    /// let mut socket = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
    /// socket.set_port(4242);
    /// assert_eq!(socket.port(), 4242);
    /// ```
    pub fn set_port(&mut self, new_port: u16) {
        self.port = new_port
    }
}
