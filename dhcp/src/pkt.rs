use w5500_hl::net::{Eui48Addr, Ipv4Addr};

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
pub struct PktSer<'a> {
    buf: &'a mut [u8],
    ptr: usize,
}

impl<'a> PktSer<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, ptr: 0 }
    }

    /// Prepares the buffer for a new DHCP message.
    ///
    /// From [RFC 2131 Section 2](https://tools.ietf.org/html/rfc2131#section-2)
    fn prepare_message(&mut self, mac: &Eui48Addr, xid: u32) -> Option<()> {
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
        self.ptr = 240;
        Some(())
    }

    /// Set the message operation code.
    fn set_op(&mut self, op: Op) -> Option<()> {
        *self.buf.get_mut(0)? = u8::from(op);
        Some(())
    }

    /// Set the hardware type to Ethernet.
    fn set_htype_ethernet(&mut self) -> Option<()> {
        *self.buf.get_mut(1)? = u8::from(HardwareType::Ethernet);
        Some(())
    }

    /// Set the hardware address length
    fn set_hlen(&mut self, len: u8) -> Option<()> {
        *self.buf.get_mut(2)? = len;
        Some(())
    }

    /// Set the hops field.
    ///
    /// Client sets to zero, optionally used by relay agents when booting via a
    /// relay agent.
    fn set_hops(&mut self, hops: u8) -> Option<()> {
        *self.buf.get_mut(3)? = hops;
        Some(())
    }

    /// Set the transaction ID.
    fn set_xid(&mut self, xid: u32) -> Option<()> {
        self.buf.get_mut(4..8)?.copy_from_slice(&xid.to_be_bytes());
        Some(())
    }

    /// Set the number of seconds elapsed since client began address acquisition
    /// or renewal process.
    fn set_secs(&mut self, secs: u16) -> Option<()> {
        self.buf
            .get_mut(8..10)?
            .copy_from_slice(&secs.to_be_bytes());
        Some(())
    }

    fn set_flags(&mut self, broadcast: bool) -> Option<()> {
        *self.buf.get_mut(10)? = (broadcast as u8) << 7;
        *self.buf.get_mut(11)? = 0;
        Some(())
    }

    /// Set the client IP address
    ///
    /// Only filled in if client is in BOUND, RENEW or REBINDING state and can
    /// respond to ARP requests.
    fn set_ciaddr(&mut self, addr: &Ipv4Addr) -> Option<()> {
        self.buf.get_mut(12..16)?.copy_from_slice(&addr.octets);
        Some(())
    }

    /// 'your' (client) IP address;
    /// filled by server if client doesn't
    /// know its own address (ciaddr was 0).
    fn set_yiaddr(&mut self, addr: &Ipv4Addr) -> Option<()> {
        self.buf.get_mut(16..20)?.copy_from_slice(&addr.octets);
        Some(())
    }

    /// Set the IP address of next server to use in bootstrap;
    /// returned in DHCPOFFER, DHCPACK by server.
    fn set_siaddr(&mut self, addr: &Ipv4Addr) -> Option<()> {
        self.buf.get_mut(20..24)?.copy_from_slice(&addr.octets);
        Some(())
    }

    /// Relay agent IP address, used in booting via a relay agent.
    fn set_giaddr(&mut self, addr: &Ipv4Addr) -> Option<()> {
        self.buf.get_mut(24..28)?.copy_from_slice(&addr.octets);
        Some(())
    }

    /// Set the hardware address
    fn set_chaddr(&mut self, mac: &Eui48Addr) -> Option<()> {
        self.buf.get_mut(28..34)?.copy_from_slice(&mac.octets);
        self.buf.get_mut(34..44)?.iter_mut().for_each(|b| *b = 0);
        Some(())
    }

    /// Set the sname field to 0
    fn set_sname_zero(&mut self) -> Option<()> {
        self.buf.get_mut(44..108)?.iter_mut().for_each(|b| *b = 0);
        Some(())
    }

    /// Set the file field to 0
    fn set_file_zero(&mut self) -> Option<()> {
        self.buf.get_mut(108..236)?.iter_mut().for_each(|b| *b = 0);
        Some(())
    }

    /// Set the magic cookie.
    ///
    /// Sets the first four octets of the options field to 99, 138, 83, 99.
    ///
    /// From [RFC 2131 Section 3](https://tools.ietf.org/html/rfc2131#section-3)
    fn set_magic_cookie(&mut self) -> Option<()> {
        const MAGIC_COOKIE: [u8; 4] = [0x63, 0x82, 0x53, 0x63];
        self.buf
            .get_mut(236..236 + MAGIC_COOKIE.len())?
            .copy_from_slice(&MAGIC_COOKIE);
        Some(())
    }

    #[inline]
    fn write_byte(&mut self, data: u8) -> Option<()> {
        *self.buf.get_mut(self.ptr)? = data;
        self.ptr += 1;
        Some(())
    }

    fn set_option_msg_type(&mut self, msg_type: MsgType) -> Option<()> {
        self.write_byte(Options::MessageType.into())?;
        self.write_byte(1)?;
        self.write_byte(msg_type.into())?;
        Some(())
    }

    fn set_option_client_id(&mut self, mac: &Eui48Addr) -> Option<()> {
        self.write_byte(Options::ClientId.into())?;
        self.write_byte(HW_ADDR_LEN + 1)?;
        self.write_byte(HardwareType::Ethernet.into())?;
        for o in mac.octets {
            self.write_byte(o)?
        }
        Some(())
    }

    fn set_option_hostname(&mut self, hostname: &str) -> Option<()> {
        let hostname_len: u8 = hostname.len().try_into().ok()?;
        self.write_byte(Options::Hostname.into())?;
        self.write_byte(hostname_len)?;
        for byte in hostname.as_bytes() {
            self.write_byte(*byte)?;
        }
        Some(())
    }

    fn set_option_parameter_request(&mut self) -> Option<()> {
        self.write_byte(Options::ParameterRequest.into());
        self.write_byte(5);
        self.write_byte(Options::SubnetMask.into());
        self.write_byte(Options::Router.into());
        self.write_byte(Options::Dns.into());
        self.write_byte(Options::RenewalTime.into());
        self.write_byte(Options::RebindingTime.into());
        Some(())
    }

    fn set_option_requested_ip(&mut self, ip: &Ipv4Addr) -> Option<()> {
        self.write_byte(Options::RequestedIp.into())?;
        self.write_byte(4)?;
        for o in ip.octets {
            self.write_byte(o)?
        }
        Some(())
    }

    fn set_option_end(&mut self) -> Option<()> {
        self.write_byte(Options::End.into())?;
        Some(())
    }

    /// Create a DHCP discover.
    pub fn dhcp_discover(&mut self, mac: &Eui48Addr, hostname: &str, xid: u32) -> Option<&[u8]> {
        self.prepare_message(mac, xid)?;
        self.set_option_msg_type(MsgType::Discover)?;
        self.set_option_client_id(mac)?;
        self.set_option_hostname(hostname)?;
        self.set_option_end()?;
        Some(&self.buf[..self.ptr])
    }

    /// Create a DHCP request.
    pub fn dhcp_request(
        &mut self,
        mac: &Eui48Addr,
        ip: &Ipv4Addr,
        hostname: &str,
        xid: u32,
    ) -> Option<&[u8]> {
        self.prepare_message(mac, xid)?;
        self.set_option_msg_type(MsgType::Request)?;
        self.set_option_client_id(mac)?;
        self.set_option_hostname(hostname)?;
        self.set_option_parameter_request()?;
        self.set_option_requested_ip(ip)?;
        self.set_option_end()?;
        Some(&self.buf[..self.ptr])
    }
}

#[derive(Debug)]
pub struct PktDe<'a> {
    buf: &'a [u8],
}

impl<'a> PktDe<'a> {
    pub fn new(buf: &'a [u8]) -> Self {
        Self { buf }
    }

    pub fn is_bootreply(&self) -> bool {
        self.buf.get(0).unwrap_or(&0).eq(&u8::from(Op::BOOTREQUEST))
    }

    pub fn xid(&self) -> Option<u32> {
        Some(u32::from_be_bytes(self.buf.get(4..8)?.try_into().unwrap()))
    }

    /// 'your' (client) IP address;
    /// filled by server if client doesn't
    /// know its own address (ciaddr was 0).
    pub fn yiaddr(&self) -> Option<Ipv4Addr> {
        let bytes: [u8; 4] = self.buf.get(16..20)?.try_into().unwrap();
        Some(bytes.into())
    }

    fn find_option_index(&self, option: Options) -> Option<usize> {
        let option: u8 = option.into();
        let mut idx: usize = 240;
        loop {
            let byte: u8 = self.buf.get(idx).copied()?;
            if byte == 0xFF {
                return None;
            } else if byte == 0x00 {
                idx += 1;
            } else if byte == option {
                return Some(idx);
            } else {
                idx += 2 + usize::from(self.buf.get(idx + 1).copied()?);
            }
        }
    }

    fn find_option_fixed_size(&self, option: Options, size: u8) -> Option<&[u8]> {
        let idx: usize = self.find_option_index(option)?;
        let option_size: u8 = self.buf.get(idx + 1).copied()?;
        if size != option_size {
            warn!(
                "malformed option {} size is {} expected {}",
                option as u8, option_size, size
            );
            None
        } else {
            Some(self.buf.get(idx + 2..idx + 2 + usize::from(size))?)
        }
    }

    fn find_option_u32(&self, option: Options) -> Option<u32> {
        let bytes: [u8; 4] = self.find_option_fixed_size(option, 4)?.try_into().unwrap();
        Some(u32::from_be_bytes(bytes))
    }

    fn find_option_ipv4(&self, option: Options) -> Option<Ipv4Addr> {
        let bytes: [u8; 4] = self.find_option_fixed_size(option, 4)?.try_into().unwrap();
        Some(bytes.into())
    }

    /// Returns the subnet mask (option 1) if it exists.
    pub fn subnet_mask(&self) -> Option<Ipv4Addr> {
        self.find_option_ipv4(Options::SubnetMask)
    }

    /// Returns the lease time (option 51) if it exists.
    pub fn lease_time(&self) -> Option<u32> {
        self.find_option_u32(Options::LeaseTime)
    }

    /// Returns the DHCP message type (option 53) if it exists.
    pub fn msg_type(&self) -> Option<MsgType> {
        let idx: usize = self.find_option_index(Options::MessageType)?;
        let size: u8 = self.buf.get(idx + 1).copied()?;
        if size != 1 {
            warn!("malformed option 53 size == {}", size);
            None
        } else {
            match MsgType::try_from(self.buf.get(idx + 2).copied()?) {
                Ok(mt) => Some(mt),
                Err(x) => {
                    warn!("not a message type value: {}", x);
                    None
                }
            }
        }
    }

    /// Returns the DHCP server identifier (option 54) if it exists
    pub fn dhcp_server(&self) -> Option<Ipv4Addr> {
        self.find_option_ipv4(Options::ServerId)
    }

    /// Returns the rebinding time (option 59) if it exists.
    pub fn rebinding_time(&self) -> Option<u32> {
        self.find_option_u32(Options::RebindingTime)
    }

    /// Returns the renewal time (option 58) if it exists.
    pub fn renewal_time(&self) -> Option<u32> {
        self.find_option_u32(Options::RenewalTime)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dhcp_discover_smoke() {
        let mut buf = vec![0u8; 1024];
        let mut pkt = PktSer {
            buf: &mut buf,
            ptr: 0,
        };

        pkt.dhcp_discover(&Eui48Addr::UNSPECIFIED, "", 0).unwrap();
    }

    #[test]
    fn dhcp_request_smoke() {
        let mut buf = vec![0u8; 1024];
        let mut pkt = PktSer {
            buf: &mut buf,
            ptr: 0,
        };

        pkt.dhcp_request(&Eui48Addr::UNSPECIFIED, &Ipv4Addr::UNSPECIFIED, "", 0)
            .unwrap();
    }
}
