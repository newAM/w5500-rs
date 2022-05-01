//! MDNS client for the [Wiznet W5500] SPI internet offload chip.

use crate::header::{Header, Qr};
#[cfg(not(feature = "std"))]
use crate::ll::net::Eui48Addr;
use crate::ll::{
    net::{Ipv4Addr, SocketAddrV4},
    Protocol, Sn, SocketCommand, SocketMode, SocketStatus,
};
use crate::{read_labels, Query, Response};
use w5500_hl::{
    io::{Read, Seek, SeekFrom},
    Error, Hostname, Udp, UdpReader, UdpWriter,
};

const MDNS_PORT: u16 = 5353;
const MDNS_ADDRESS: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
const MDNS_SERVER: SocketAddrV4 = SocketAddrV4::new(MDNS_ADDRESS, MDNS_PORT);
#[cfg(not(feature = "std"))]
const MDNS_HARDWARE_DST: Eui48Addr = Eui48Addr::new(0x01, 0x00, 0x5E, 0x00, 0x00, 0xFB);

/// W5500 MDNS client.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Client {
    sn: Sn,
    port: u16,
    server: SocketAddrV4,
}

impl Client {
    /// Create a new MDNS client.
    ///
    /// # Arguments
    ///
    /// * `sn` - The socket number to use for MDNS queries.
    /// * `port` - MDNS source port.
    ///   Use any free port greater than 1024 not in use by other W5500 sockets or None
    ///   for the default MDNS port of 5353
    ///
    /// # Example
    ///
    /// ```
    /// use w5500_dns::{ll::Sn, mdns::Client as MdnsClient};
    ///
    /// const MDNS_SRC_PORT: u16 = 45917;
    ///
    /// let mdns_client: MdnsClient = MdnsClient::new(Sn::Sn3, Some(MDNS_SRC_PORT));
    /// ```
    #[must_use]
    pub fn new(sn: Sn, port: Option<u16>) -> Self {
        Self {
            sn,
            port: port.unwrap_or(MDNS_PORT),
            server: MDNS_SERVER,
        }
    }

    /// A simple MDNS query.
    ///
    /// This will only broadcast an MDNS query, it will not wait for a reply.
    fn query<'a, W5500: Udp>(
        &mut self,
        w5500: &'a mut W5500,
    ) -> Result<Query<'a, W5500>, Error<W5500::Error>> {
        w5500.set_sn_dhar(self.sn, &MDNS_HARDWARE_DST)?;
        w5500.set_sn_ttl(self.sn, 255)?;
        w5500.set_sn_cr(self.sn, SocketCommand::Close)?;
        while w5500.sn_sr(self.sn)? != Ok(SocketStatus::Closed) {}
        w5500.set_sn_port(self.sn, self.port)?;
        const MODE: SocketMode = SocketMode::DEFAULT
            .set_protocol(Protocol::Udp)
            .enable_multi();
        w5500.set_sn_mr(self.sn, MODE)?;
        w5500.set_sn_cr(self.sn, SocketCommand::Open)?;
        // This will not hang, the socket status will always change to Udp
        // after a open command with SN_MR set to UDP.
        // (unless you do somthing silly like holding the W5500 in reset)
        while w5500.sn_sr(self.sn)? != Ok(SocketStatus::Udp) {}
        w5500.set_sn_dest(self.sn, &self.server)?;
        const HEADER_SEEK: SeekFrom = SeekFrom::Start(Header::LEN);
        let mut writer: UdpWriter<W5500> = w5500.udp_writer(self.sn)?;
        writer.seek(HEADER_SEEK)?;
        Ok(Query {
            writer,
            header: Header::new_query(0),
        })
    }

    /// Send an MDNS A record query.
    ///
    /// This will only broadcst an MDNS A record query, it will not wait for any replies from
    /// MDNS responders.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::OutOfMemory`]
    pub fn a_question<'a, W5500: Udp>(
        &mut self,
        w5500: &'a mut W5500,
        hostname: &Hostname,
    ) -> Result<(), Error<W5500::Error>> {
        self.query(w5500)?.question(hostname)?.send()?;
        Ok(())
    }

    /// Retrieve MDNS broadcast traffic.
    ///
    /// The nature of MDNS is such that traffic unrelated to
    /// any query made on this client can appear here.
    ///
    /// # Arguments
    ///
    /// * `w5500`: The W5500 device that implements the [`Udp`] trait.
    /// * `buf`: A buffer for reading the hostname.
    ///   Hostnames can be up to 255 bytes.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::UnexpectedEof`]
    /// * [`Error::WouldBlock`]
    pub fn response<'a, W5500: Udp>(
        &self,
        w5500: &'a mut W5500,
        buf: &'a mut [u8],
    ) -> Result<Response<'a, W5500>, Error<W5500::Error>> {
        let mut reader: UdpReader<W5500> = w5500.udp_reader(self.sn)?;

        let mut dns_header_buf = Header::new_buf();
        let n: u16 = reader.read(&mut dns_header_buf)?;
        if n != Header::LEN {
            reader.done()?;
            return Err(Error::WouldBlock);
        }

        let header: Header = dns_header_buf.into();

        if header.qr() != Qr::Response {
            reader.done()?;
            return Err(Error::WouldBlock);
        }

        // ignore all the questions
        for _ in 0..header.qdcount() {
            // seek over labels
            read_labels(&mut reader, &mut [])?;

            // seek over type and class
            reader.seek(SeekFrom::Current(4))?;
        }

        Ok(Response {
            reader,
            header,
            buf,
            answer_idx: 0,
        })
    }
}
