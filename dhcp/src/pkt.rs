use w5500_hl::{
    ll::{Registers, Sn},
    net::{Eui48Addr, Ipv4Addr},
    Common, Error, Hostname, Read, Seek, SeekFrom, UdpReader, Writer,
};

/// DHCP options.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[allow(dead_code)]
enum Options {
    Pad = 0,
    SubnetMask = 1,
    TimeOffset = 2,
    Router = 3,
    TimeServer = 4,
    NameServer = 5,
    Dns = 6,
    Hostname = 12,
    /// Requested IP Address
    ///
    /// From [RFC 2132 Section 9.1](https://tools.ietf.org/html/rfc2132#section-9.1)
    RequestedIp = 50,
    LeaseTime = 51,
    MessageType = 53,
    ServerId = 54,
    ParameterRequest = 55,
    RenewalTime = 58,
    RebindingTime = 59,
    /// Client-identifier
    ///
    /// From [RFC 2132 Section 9.14](https://tools.ietf.org/html/rfc2132#section-9.14)
    ClientId = 61,
    End = 255,
}
impl From<Options> for u8 {
    fn from(val: Options) -> u8 {
        val as u8
    }
}

/// DHCP message types.
///
/// From [RFC 2132 Section 9.6](https://tools.ietf.org/html/rfc2132#section-9.6)
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MsgType {
    /// DHCPDISCOVER
    Discover = 1,
    /// DHCPOFFER
    Offer = 2,
    /// DHCPREQUEST
    Request = 3,
    /// DHCPDECLINE
    Decline = 4,
    /// DHCPACK
    Ack = 5,
    /// DHCPNAK
    Nak = 6,
    /// DHCPRELEASE
    Release = 7,
    /// DHCPINFORM
    Inform = 8,
}

impl From<MsgType> for u8 {
    fn from(val: MsgType) -> u8 {
        val as u8
    }
}

impl TryFrom<u8> for MsgType {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            x if x == MsgType::Discover as u8 => Ok(MsgType::Discover),
            x if x == MsgType::Offer as u8 => Ok(MsgType::Offer),
            x if x == MsgType::Request as u8 => Ok(MsgType::Request),
            x if x == MsgType::Decline as u8 => Ok(MsgType::Decline),
            x if x == MsgType::Ack as u8 => Ok(MsgType::Ack),
            x if x == MsgType::Nak as u8 => Ok(MsgType::Nak),
            x if x == MsgType::Release as u8 => Ok(MsgType::Release),
            x if x == MsgType::Inform as u8 => Ok(MsgType::Inform),
            x => Err(x),
        }
    }
}

/// DHCP op code (message type)
///
/// From [RFC 2131 Section 2](https://tools.ietf.org/html/rfc2131#section-2)
#[repr(u8)]
#[allow(clippy::upper_case_acronyms)]
pub enum Op {
    BOOTREQUEST = 1,
    BOOTREPLY = 2,
}

impl From<Op> for u8 {
    fn from(val: Op) -> u8 {
        val as u8
    }
}

/// DHCP hardware type.
///
/// See [RFC 1700](https://tools.ietf.org/html/rfc1700)
#[repr(u8)]
#[non_exhaustive]
pub enum HardwareType {
    Ethernet = 1,
    // lots of others that we do not need to care about
}
impl From<HardwareType> for u8 {
    fn from(val: HardwareType) -> u8 {
        val as u8
    }
}

const HW_ADDR_LEN: u8 = 6;

#[derive(Debug)]
struct PktSer<'a, W: Registers> {
    writer: Writer<'a, W>,
}

impl<'a, W: Registers> From<Writer<'a, W>> for PktSer<'a, W> {
    fn from(writer: Writer<'a, W>) -> Self {
        Self { writer }
    }
}

impl<'a, W: Registers> PktSer<'a, W> {
    /// Prepares the buffer for a new DHCP message.
    ///
    /// From [RFC 2131 Section 2](https://tools.ietf.org/html/rfc2131#section-2)
    fn prepare_message(&mut self, mac: &Eui48Addr, xid: u32) -> Result<(), Error<W::Error>> {
        // self.zero();
        self.set_op(Op::BOOTREQUEST)?;
        self.set_htype_ethernet()?;
        self.set_hlen(HW_ADDR_LEN)?;
        self.set_hops(0)?;
        self.set_xid(xid)?;
        self.set_secs(0)?;
        self.set_flags(true)?;
        self.set_ciaddr(&Ipv4Addr::UNSPECIFIED)?;
        self.set_yiaddr(&Ipv4Addr::UNSPECIFIED)?;
        self.set_siaddr(&Ipv4Addr::UNSPECIFIED)?;
        self.set_giaddr(&Ipv4Addr::UNSPECIFIED)?;
        self.set_chaddr(mac)?;
        self.set_sname_zero()?;
        self.set_file_zero()?;
        self.set_magic_cookie()?;
        self.writer.seek(SeekFrom::Start(240));

        Ok(())
    }

    /// Set the message operation code.
    fn set_op(&mut self, op: Op) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(0));
        self.writer.write_all(&[u8::from(op)])
    }

    /// Set the hardware type to Ethernet.
    fn set_htype_ethernet(&mut self) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(1));
        self.writer.write_all(&[u8::from(HardwareType::Ethernet)])
    }

    /// Set the hardware address length
    fn set_hlen(&mut self, len: u8) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(2));
        self.writer.write_all(&[len])
    }

    /// Set the hops field.
    ///
    /// Client sets to zero, optionally used by relay agents when booting via a
    /// relay agent.
    fn set_hops(&mut self, hops: u8) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(3));
        self.writer.write_all(&[hops])
    }

    /// Set the transaction ID.
    fn set_xid(&mut self, xid: u32) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(4));
        self.writer.write_all(&xid.to_be_bytes())
    }

    /// Set the number of seconds elapsed since client began address acquisition
    /// or renewal process.
    fn set_secs(&mut self, secs: u16) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(8));
        self.writer.write_all(&secs.to_be_bytes())
    }

    fn set_flags(&mut self, broadcast: bool) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(10));
        self.writer.write_all(&[(broadcast as u8) << 7, 0])
    }

    /// Set the client IP address
    ///
    /// Only filled in if client is in BOUND, RENEW or REBINDING state and can
    /// respond to ARP requests.
    fn set_ciaddr(&mut self, addr: &Ipv4Addr) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(12));
        self.writer.write_all(&addr.octets)
    }

    /// 'your' (client) IP address;
    /// filled by server if client doesn't
    /// know its own address (ciaddr was 0).
    fn set_yiaddr(&mut self, addr: &Ipv4Addr) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(16));
        self.writer.write_all(&addr.octets)
    }

    /// Set the IP address of next server to use in bootstrap;
    /// returned in DHCPOFFER, DHCPACK by server.
    fn set_siaddr(&mut self, addr: &Ipv4Addr) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(20));
        self.writer.write_all(&addr.octets)
    }

    /// Relay agent IP address, used in booting via a relay agent.
    fn set_giaddr(&mut self, addr: &Ipv4Addr) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(24));
        self.writer.write_all(&addr.octets)
    }

    /// Set the hardware address
    fn set_chaddr(&mut self, mac: &Eui48Addr) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(28));
        self.writer.write_all(&mac.octets)?;
        let zeros: [u8; 10] = [0; 10];
        self.writer.write_all(&zeros)
    }

    /// Set the sname field to 0
    fn set_sname_zero(&mut self) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(44));
        let buf: [u8; 64] = [0; 64]; // perhaps a bit much for the stack?
        self.writer.write_all(&buf)
    }

    /// Set the file field to 0
    fn set_file_zero(&mut self) -> Result<(), Error<W::Error>> {
        self.writer.seek(SeekFrom::Start(108));
        let buf: [u8; 64] = [0; 64]; // perhaps a bit much for the stack?
                                     // needs 128 bytes, write it twice
        self.writer.write_all(&buf)?;
        self.writer.write_all(&buf)
    }

    /// Set the magic cookie.
    ///
    /// Sets the first four octets of the options field to 99, 138, 83, 99.
    ///
    /// From [RFC 2131 Section 3](https://tools.ietf.org/html/rfc2131#section-3)
    fn set_magic_cookie(&mut self) -> Result<(), Error<W::Error>> {
        const MAGIC_COOKIE: [u8; 4] = [0x63, 0x82, 0x53, 0x63];
        self.writer.seek(SeekFrom::Start(236));
        self.writer.write_all(&MAGIC_COOKIE)
    }

    fn set_option_msg_type(&mut self, msg_type: MsgType) -> Result<(), Error<W::Error>> {
        self.writer
            .write_all(&[Options::MessageType.into(), 1, msg_type.into()])
    }

    fn set_option_client_id(&mut self, mac: &Eui48Addr) -> Result<(), Error<W::Error>> {
        self.writer.write_all(&[
            Options::ClientId.into(),
            HW_ADDR_LEN + 1,
            HardwareType::Ethernet.into(),
        ])?;
        self.writer.write_all(&mac.octets)
    }

    fn set_option_hostname(&mut self, hostname: Hostname) -> Result<(), Error<W::Error>> {
        let hostname_len: u8 = hostname.len();
        self.writer
            .write_all(&[Options::Hostname.into(), hostname_len])?;
        self.writer.write_all(hostname.as_bytes())
    }

    fn set_option_parameter_request(&mut self) -> Result<(), Error<W::Error>> {
        self.writer.write_all(&[
            Options::ParameterRequest.into(),
            5,
            Options::SubnetMask.into(),
            Options::Router.into(),
            Options::Dns.into(),
            Options::RenewalTime.into(),
            Options::RebindingTime.into(),
        ])
    }

    fn set_option_requested_ip(&mut self, ip: &Ipv4Addr) -> Result<(), Error<W::Error>> {
        self.writer.write_all(&[Options::RequestedIp.into(), 4])?;
        self.writer.write_all(&ip.octets)
    }

    fn set_option_end(&mut self) -> Result<(), Error<W::Error>> {
        self.writer.write_all(&[Options::End.into()])
    }

    /// Create a DHCP discover.
    fn dhcp_discover(
        mut self,
        mac: &Eui48Addr,
        hostname: Hostname,
        xid: u32,
    ) -> Result<Writer<'a, W>, Error<W::Error>> {
        self.prepare_message(mac, xid)?;
        self.set_option_msg_type(MsgType::Discover)?;
        self.set_option_client_id(mac)?;
        self.set_option_hostname(hostname)?;
        self.set_option_end()?;
        Ok(self.writer)
    }

    /// Create a DHCP request.
    fn dhcp_request(
        mut self,
        mac: &Eui48Addr,
        ip: &Ipv4Addr,
        hostname: Hostname,
        xid: u32,
    ) -> Result<Writer<'a, W>, Error<W::Error>> {
        self.prepare_message(mac, xid)?;
        self.set_option_msg_type(MsgType::Request)?;
        self.set_option_client_id(mac)?;
        self.set_option_hostname(hostname)?;
        self.set_option_parameter_request()?;
        self.set_option_requested_ip(ip)?;
        self.set_option_end()?;
        Ok(self.writer)
    }
}

pub fn send_dhcp_discover<W: Registers>(
    w5500: &mut W,
    sn: Sn,
    mac: &Eui48Addr,
    hostname: Hostname,
    xid: u32,
) -> Result<(), Error<W::Error>> {
    let writer: Writer<W> = w5500.writer(sn)?;
    PktSer::from(writer)
        .dhcp_discover(mac, hostname, xid)?
        .udp_send_to(&crate::DHCP_BROADCAST)?;
    Ok(())
}

pub fn send_dhcp_request<W: Registers>(
    w5500: &mut W,
    sn: Sn,
    mac: &Eui48Addr,
    ip: &Ipv4Addr,
    hostname: Hostname,
    xid: u32,
) -> Result<(), Error<W::Error>> {
    let writer: Writer<W> = w5500.writer(sn)?;
    PktSer::from(writer)
        .dhcp_request(mac, ip, hostname, xid)?
        .send()?;
    Ok(())
}

#[derive(Debug)]
pub struct PktDe<'a, W: Registers> {
    reader: UdpReader<'a, W>,
}

impl<'a, W: Registers> From<UdpReader<'a, W>> for PktDe<'a, W> {
    fn from(reader: UdpReader<'a, W>) -> Self {
        Self { reader }
    }
}

impl<'a, W: Registers> PktDe<'a, W> {
    #[allow(clippy::wrong_self_convention)]
    pub fn is_bootreply(&mut self) -> Result<bool, Error<W::Error>> {
        self.reader.seek(SeekFrom::Start(0));
        let mut buf: [u8; 1] = [0];
        self.reader.read_exact(&mut buf)?;
        Ok(buf[0] == u8::from(Op::BOOTREQUEST))
    }

    pub fn xid(&mut self) -> Result<u32, Error<W::Error>> {
        self.reader.seek(SeekFrom::Start(4));
        let mut buf: [u8; 4] = [0; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    /// 'your' (client) IP address;
    /// filled by server if client doesn't
    /// know its own address (ciaddr was 0).
    pub fn yiaddr(&mut self) -> Result<Ipv4Addr, Error<W::Error>> {
        self.reader.seek(SeekFrom::Start(16));
        let mut buf: [u8; 4] = [0; 4];
        self.reader.read_exact(&mut buf)?;
        Ok(buf.into())
    }

    /// Seeks to an option and returns the length if it exists.
    fn seek_to_option(&mut self, option: Options) -> Result<Option<u8>, Error<W::Error>> {
        let option: u8 = option.into();
        self.reader.seek(SeekFrom::Start(240));
        loop {
            let (byte, len): (u8, u8) = {
                let mut buf: [u8; 2] = [0; 2];
                self.reader.read_exact(&mut buf)?;
                (buf[0], buf[1])
            };
            if byte == 0xFF {
                return Ok(None);
            } else if byte == 0x00 {
                if len == 0x00 {
                    continue;
                } else {
                    self.reader.seek(SeekFrom::Current(-1))
                }
            } else if byte == option {
                return Ok(Some(len));
            } else {
                self.reader.seek(SeekFrom::Current(len.into()));
            }
        }
    }

    fn find_option_fixed_size<const N: usize>(
        &mut self,
        option: Options,
    ) -> Result<Option<[u8; N]>, Error<W::Error>> {
        let option_size: u8 = match self.seek_to_option(option)? {
            Some(len) => len,
            None => return Ok(None),
        };
        if usize::from(option_size) != N {
            warn!(
                "malformed option {} size is {} expected {}",
                option as u8, option_size, N
            );
            Ok(None)
        } else {
            let mut buf: [u8; N] = [0; N];
            self.reader.read_exact(&mut buf)?;
            Ok(Some(buf))
        }
    }

    fn find_option_u32(&mut self, option: Options) -> Result<Option<u32>, Error<W::Error>> {
        match self.find_option_fixed_size(option)? {
            Some(bytes) => Ok(Some(u32::from_be_bytes(bytes))),
            None => Ok(None),
        }
    }

    fn find_option_ipv4(&mut self, option: Options) -> Result<Option<Ipv4Addr>, Error<W::Error>> {
        match self.find_option_fixed_size(option)? {
            Some(bytes) => Ok(Some(bytes.into())),
            None => Ok(None),
        }
    }

    /// Returns the subnet mask (option 1) if it exists.
    pub fn subnet_mask(&mut self) -> Result<Option<Ipv4Addr>, Error<W::Error>> {
        self.find_option_ipv4(Options::SubnetMask)
    }

    /// Returns the lease time (option 51) if it exists.
    pub fn lease_time(&mut self) -> Result<Option<u32>, Error<W::Error>> {
        self.find_option_u32(Options::LeaseTime)
    }

    /// Returns the DHCP message type (option 53) if it exists.
    pub fn msg_type(&mut self) -> Result<Option<MsgType>, Error<W::Error>> {
        let buf: [u8; 1] = match self.find_option_fixed_size(Options::MessageType)? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };
        match MsgType::try_from(buf[0]) {
            Ok(mt) => Ok(Some(mt)),
            Err(x) => {
                warn!("not a message type value: {}", x);
                Ok(None)
            }
        }
    }

    /// Returns the DHCP server identifier (option 54) if it exists
    pub fn dhcp_server(&mut self) -> Result<Option<Ipv4Addr>, Error<W::Error>> {
        self.find_option_ipv4(Options::ServerId)
    }

    /// Returns the rebinding time (option 59) if it exists.
    pub fn rebinding_time(&mut self) -> Result<Option<u32>, Error<W::Error>> {
        self.find_option_u32(Options::RebindingTime)
    }

    /// Returns the renewal time (option 58) if it exists.
    pub fn renewal_time(&mut self) -> Result<Option<u32>, Error<W::Error>> {
        self.find_option_u32(Options::RenewalTime)
    }

    pub fn done(self) -> Result<(), W::Error> {
        self.reader.done()
    }
}
