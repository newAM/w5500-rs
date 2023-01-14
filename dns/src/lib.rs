//! DNS client for the [Wiznet W5500] SPI internet offload chip.
//!
//! # Warning
//!
//! Please proceed with caution, and review the code before use in a production
//! environment.
//!
//! This code was developed for one-off hobby projects.
//!
//! # Limitations
//!
//! * No DNS caching.
//! * Only supports A queries.
//! * Only supports a single outstanding query.
//! * Only supports a single question in a query.
//!
//! # Example
//!
//! ```no_run
//! # let mut w5500 = w5500_regsim::W5500::default();
//! # let random_number: u64 = 0;
//! use w5500_dns::{hl::block, ll::Sn, servers, Client as DnsClient, Hostname, Response};
//!
//! const DNS_SOCKET: Sn = Sn::Sn3;
//! const DNS_SRC_PORT: u16 = 45917;
//!
//! let mut dns_client: DnsClient =
//!     DnsClient::new(DNS_SOCKET, DNS_SRC_PORT, servers::CLOUDFLARE, random_number);
//! let hostname: Hostname = Hostname::new("docs.rs").expect("hostname is invalid");
//!
//! let mut hostname_buffer: [u8; 16] = [0; 16];
//!
//! let query_id: u16 = dns_client.a_question(&mut w5500, &hostname)?;
//! let mut response: Response<_> =
//!     block!(dns_client.response(&mut w5500, &mut hostname_buffer, query_id))?;
//!
//! while let Some(rr) = response.next_rr()? {
//!     println!("name: {:?}", rr.name);
//!     println!("TTL: {}", rr.ttl);
//!     println!("IP: {:?}", rr.rdata);
//! }
//! response.done()?;
//! # Ok::<(), w5500_hl::Error<std::io::ErrorKind>>(())
//! ```
//!
//! # Relevant Specifications
//!
//! * [RFC 1035](https://www.rfc-editor.org/rfc/rfc1035)
//!
//! # Feature Flags
//!
//! All features are disabled by default.
//!
//! * `eh0`: Passthrough to [w5500-hl].
//! * `eh1`: Passthrough to [w5500-hl].
//! * `std`: Passthrough to [w5500-hl].
//! * `defmt`: Enable logging with `defmt`. Also a passthrough to [w5500-hl].
//! * `log`: Enable logging with `log`.
//!
//! [w5500-hl]: https://crates.io/crates/w5500-hl
//! [`std::net`]: https://doc.rust-lang.org/std/net/index.html
//! [Wiznet W5500]: https://www.wiznet.io/product-item/w5500/
#![cfg_attr(docsrs, feature(doc_cfg), feature(doc_auto_cfg))]
#![cfg_attr(all(not(feature = "std"), not(test)), no_std)]
#![deny(unsafe_code)]
#![warn(missing_docs)]

// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod header;
pub mod mdns;
mod qclass;
mod qtype;
mod rand;

pub use header::ResponseCode;
use header::{Header, Qr};
pub use hl::Hostname;
use hl::{
    io::{Read, Seek, SeekFrom, Write},
    Error, Udp, UdpReader, UdpWriter,
};
use ll::{
    net::{Ipv4Addr, SocketAddrV4},
    Sn,
};
pub use qclass::Qclass;
pub use qtype::Qtype;
pub use w5500_hl as hl;
pub use w5500_hl::ll;

/// DNS destination port.
pub const DST_PORT: u16 = 53;

/// Commonly used public DNS servers.
///
/// When using DHCP it is typically a good idea to use the DNS server provided
/// by the DHCP server.
pub mod servers {
    use super::Ipv4Addr;

    /// Cloudflare's public DNS.
    ///
    /// More information: <https://www.cloudflare.com/en-gb/learning/dns/what-is-1.1.1.1/>
    pub const CLOUDFLARE: Ipv4Addr = Ipv4Addr::new(1, 1, 1, 1);
    /// Google's public DNS.
    ///
    /// More information: <https://developers.google.com/speed/public-dns>
    pub const GOOGLE_1: Ipv4Addr = Ipv4Addr::new(8, 8, 8, 8);
    /// Google's alternate public DNS.
    ///
    /// More information: <https://developers.google.com/speed/public-dns>
    pub const GOOGLE_2: Ipv4Addr = Ipv4Addr::new(8, 8, 4, 4);
}

/// Decoded fields from SRV rdata.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ServiceData {
    /// The priority of this target host.
    pub priority: u16,
    /// A server selection mechanism.
    pub weight: u16,
    /// The port on this target host of this service.
    pub port: u16,
}

/// Decoded rdata.
#[derive(Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RData {
    /// Decoded rdata for A records.
    A(Ipv4Addr),
    /// Decoded rdata for SVR records.
    SVR(ServiceData),
    /// Record type's rdata is unsupported.
    #[default]
    Unsupported,
}

/// DNS server resource records.
///
/// This is created by [`Response::next_rr`].
///
/// # References
///
/// * [RFC 1035 Section 3.2](https://datatracker.ietf.org/doc/html/rfc1035#section-3.2)
/// * [RFC 1035 Section 4.1.4](https://www.rfc-editor.org/rfc/rfc1035#section-4.1.4)
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ResourceRecord<'a> {
    /// A domain name to which this resource record pertains.
    ///
    /// This will be `None` if the domain name contains invalid characters or if
    /// the provided buffer was not large enough to contain the entire name.
    pub name: Option<&'a str>,
    /// Resource record type.
    ///
    /// If the value from the DNS server does not match a valid [`Qtype`] the
    /// value will be returned in the `Err` variant.
    pub qtype: Result<Qtype, u16>,
    /// Resource record class.
    ///
    /// Only internet records are supported at the moment, which means we can
    /// assume this is `Ok(Qclass::IN)` if the DNS server is operating correctly.
    ///
    /// If the value from the DNS server does not match a valid [`Qtype`] the
    /// value will be returned in the `Err` variant.
    pub class: Result<Qclass, u16>,
    /// Time to live.
    ///
    /// The time interval that the resource record may be cached before the
    /// source of the information should again be consulted.
    /// Zero values are interpreted to mean that the RR can only be used for the
    /// transaction in progress, and should not be cached.
    /// For example, SOA records are always distributed with a zero TTL to
    /// prohibit caching.  Zero values can also be used for extremely volatile
    /// data.
    pub ttl: u32,
    /// Resource record data.
    ///
    /// Only the RDATA in A and SVR records is supported at the moment.
    ///
    /// This is `RData::Unsupported` if the rdata is not from a supported record type.
    pub rdata: RData,
}

// https://www.rfc-editor.org/rfc/rfc1035#section-4.1.4
fn read_labels<'l, E, Reader: Read<E> + Seek>(
    reader: &mut Reader,
    labels: &'l mut [u8],
) -> Result<Option<&'l str>, Error<E>> {
    const NAME_PTR_MASK: u16 = 0xC0_00;

    let mut labels_idx: usize = 0;
    let mut seek_to: u16 = 0;

    loop {
        let mut buf: [u8; 2] = [0; 2];
        let n: u16 = reader.read(&mut buf)?;
        if n == 0 {
            return Err(Error::UnexpectedEof);
        }

        let sequence: u16 = u16::from_be_bytes(buf);

        // if pointer
        if n == 2 && sequence & NAME_PTR_MASK != 0 {
            let ptr: u16 = sequence & !NAME_PTR_MASK;
            if seek_to == 0 {
                seek_to = reader.stream_position();
            }
            reader.seek(SeekFrom::Start(ptr))?;
        } else {
            // seek back to the start of the label
            reader.seek(SeekFrom::Current(-1))?;

            let len: u8 = buf[0] & 0x3F;
            if len == 0 {
                break;
            }

            if labels_idx != 0 {
                if let Some(b) = labels.get_mut(labels_idx) {
                    *b = b'.';
                }
                labels_idx += 1;
            }

            if let Some(label_buf) = labels.get_mut(labels_idx..(labels_idx + usize::from(len))) {
                reader.read_exact(label_buf)?;
            } else {
                reader.seek(SeekFrom::Current(len.into()))?;
            }
            labels_idx += usize::from(len);
        }
    }

    if seek_to != 0 {
        reader.seek(SeekFrom::Start(seek_to))?;
    }

    if let Some(name_buf) = labels.get(..labels_idx) {
        Ok(core::str::from_utf8(name_buf).ok())
    } else {
        Ok(None)
    }
}

fn read_u16<E, Reader: Read<E>>(reader: &mut Reader) -> Result<u16, Error<E>> {
    let v: u16 = {
        let mut buf: [u8; 2] = [0; core::mem::size_of::<u16>()];
        reader.read_exact(&mut buf)?;
        u16::from_be_bytes(buf)
    };
    Ok(v)
}

fn read_u32<E, Reader: Read<E>>(reader: &mut Reader) -> Result<u32, Error<E>> {
    let v: u32 = {
        let mut buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut buf)?;
        u32::from_be_bytes(buf)
    };
    Ok(v)
}

/// DNS response from the server.
///
/// This is created with [`Client::response`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Response<'a, W5500: Udp> {
    reader: UdpReader<'a, W5500>,
    header: Header,
    buf: &'a mut [u8],
    rr_idx: u16,
}

impl<'a, W5500: Udp> Response<'a, W5500> {
    /// Response code from the server.
    ///
    /// This will return `Err(u8)` if the server uses a reserved value.
    pub fn response_code(&self) -> Result<ResponseCode, u8> {
        self.header.rcode()
    }

    /// Number of answers in the response.
    #[must_use]
    pub fn answer_count(&self) -> u16 {
        self.header.ancount()
    }

    /// Number of resource records in the response.
    #[must_use]
    pub fn rr_count(&self) -> u16 {
        self.header
            .ancount()
            .saturating_add(self.header.nscount())
            .saturating_add(self.header.arcount())
    }

    /// Get the next resource record from the DNS response.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::UnexpectedEof`]
    ///
    /// If any error occurs the entire response should be discarded,
    /// and you should not continue to call `next_rr`.
    pub fn next_rr(&mut self) -> Result<Option<ResourceRecord>, Error<W5500::Error>> {
        if self.rr_idx >= self.rr_count() {
            Ok(None)
        } else {
            self.rr_idx = self.rr_idx.saturating_add(1);

            let name: Option<&str> = read_labels(&mut self.reader, self.buf)?;

            let qtype = read_u16(&mut self.reader)?;
            let class = read_u16(&mut self.reader)?;
            let ttl = read_u32(&mut self.reader)?;
            let rdlength = read_u16(&mut self.reader)?;

            let qtype = qtype.try_into();

            let before_rdata_pos = self.reader.stream_position();

            let rdata = match qtype {
                Ok(Qtype::A) => {
                    let mut buf: [u8; 4] = [0; 4];
                    self.reader.read_exact(&mut buf)?;
                    RData::A(Ipv4Addr::from(buf))
                }
                Ok(Qtype::SVR) => {
                    let priority = read_u16(&mut self.reader)?;
                    let weight = read_u16(&mut self.reader)?;
                    let port = read_u16(&mut self.reader)?;
                    RData::SVR(ServiceData {
                        priority,
                        weight,
                        port,
                    })
                }
                _ => RData::Unsupported,
            };

            if rdlength != 0 {
                self.reader
                    .seek(SeekFrom::Start(before_rdata_pos.saturating_add(rdlength)))?;
            }

            // now we are at the rest of the answer.
            Ok(Some(ResourceRecord {
                name,
                qtype,
                class: class.try_into(),
                ttl,
                rdata,
            }))
        }
    }

    /// Mark this response as done, removing it from the queue.
    ///
    /// If this is not called the same response will appear on the next
    /// call to [`Client::response`].
    pub fn done(self) -> Result<(), W5500::Error> {
        self.reader.done()
    }
}

/// DNS query sent by the client.
///
/// This is created with [`Client::query`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
struct Query<'a, W5500: Udp> {
    writer: UdpWriter<'a, W5500>,
    header: Header,
}

impl<'a, W5500: Udp> Query<'a, W5500> {
    /// Add a question to the query.
    ///
    /// # References
    ///
    /// * [RFC 1035 Section 4.1.2](https://www.rfc-editor.org/rfc/rfc1035#section-4.1.2)
    pub fn question(mut self, qtype: Qtype, qname: &Hostname) -> Result<Self, Error<W5500::Error>> {
        const REMAIN_LEN: u16 = 5;

        if self.writer.remain() < u16::from(qname.len()) + REMAIN_LEN {
            return Err(Error::OutOfMemory);
        }

        for label in qname.labels() {
            // truncation from usize to u8 will not loose precision
            // Hostname is validated to have labels with 63 or fewer bytes
            let label_len: u8 = label.len() as u8;

            self.writer.write_all(&[label_len])?;
            self.writer.write_all(label.as_bytes())?;
        }

        let question_remainder: [u8; REMAIN_LEN as usize] = [
            0, // null terminator for above labels
            qtype.high_byte(),
            qtype.low_byte(),
            Qclass::IN.high_byte(),
            Qclass::IN.low_byte(),
        ];

        self.writer.write_all(&question_remainder)?;

        self.header.increment_qdcount();

        Ok(self)
    }

    /// Send the DNS query.
    pub fn send(mut self) -> Result<u16, Error<W5500::Error>> {
        if self.header.qdcount() == 0 {
            Ok(self.header.id())
        } else {
            let restore: u16 = self.writer.stream_position();
            self.writer.rewind();
            self.writer.write_all(self.header.as_bytes())?;
            self.writer.seek(SeekFrom::Start(restore))?;
            self.writer.send()?;
            Ok(self.header.id())
        }
    }
}

/// W5500 DNS client.
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Client {
    sn: Sn,
    port: u16,
    server: SocketAddrV4,
    rng: rand::Rand,
}

impl Client {
    /// Create a new DNS client.
    ///
    /// # Arguments
    ///
    /// * `sn` - The socket number to use for DNS queries.
    /// * `port` - DNS source port.
    ///   Use any free port greater than 1024 not in use by other W5500 sockets.
    /// * `server` - The DNS server IPv4 address.
    ///   Typically this is a DNS server provided by your DHCP client, but you
    ///   can also use any server from the [`servers`] module.
    /// * `seed` - A random `u64` to seed the random number generator used to
    ///   create transaction IDs.
    ///
    /// # Example
    ///
    /// ```
    /// # let random_number: u64 = 0;
    /// use w5500_dns::{ll::Sn, servers, Client};
    ///
    /// const DNS_SRC_PORT: u16 = 45917;
    ///
    /// let dns_client: Client = Client::new(Sn::Sn3, DNS_SRC_PORT, servers::CLOUDFLARE, random_number);
    /// ```
    #[must_use]
    pub const fn new(sn: Sn, port: u16, server: Ipv4Addr, seed: u64) -> Self {
        Self {
            sn,
            port,
            server: SocketAddrV4::new(server, DST_PORT),
            rng: rand::Rand::new(seed),
        }
    }

    /// Set the DNS server.
    ///
    /// # Example
    ///
    /// ```
    /// # let random_number: u64 = 0;
    /// use w5500_dns::{ll::Sn, servers, Client};
    ///
    /// const DNS_SRC_PORT: u16 = 45917;
    ///
    /// let mut dns_client: Client =
    ///     Client::new(Sn::Sn3, DNS_SRC_PORT, servers::CLOUDFLARE, random_number);
    /// assert_eq!(dns_client.server(), servers::CLOUDFLARE);
    ///
    /// dns_client.set_server(servers::GOOGLE_1);
    /// assert_eq!(dns_client.server(), servers::GOOGLE_1);
    /// ```
    #[inline]
    pub fn set_server(&mut self, server: Ipv4Addr) {
        self.server.set_ip(server)
    }

    /// Get the current DNS server.
    ///
    /// # Example
    ///
    /// ```
    /// # let random_number: u64 = 0;
    /// use w5500_dns::{ll::Sn, servers, Client};
    ///
    /// const DNS_SRC_PORT: u16 = 45917;
    ///
    /// let dns_client: Client = Client::new(Sn::Sn3, DNS_SRC_PORT, servers::CLOUDFLARE, random_number);
    /// assert_eq!(dns_client.server(), servers::CLOUDFLARE);
    /// ```
    #[inline]
    #[must_use]
    pub fn server(&self) -> Ipv4Addr {
        *self.server.ip()
    }

    /// A simple DNS query.
    ///
    /// This will only send a DNS or MDNS query, it will not wait for a reply.
    fn query<'a, W5500: Udp>(
        &mut self,
        w5500: &'a mut W5500,
    ) -> Result<Query<'a, W5500>, Error<W5500::Error>> {
        w5500.udp_bind(self.sn, self.port)?;
        w5500.set_sn_dest(self.sn, &self.server)?;
        const HEADER_SEEK: SeekFrom = SeekFrom::Start(Header::LEN);
        let mut writer: UdpWriter<W5500> = w5500.udp_writer(self.sn)?;
        writer.seek(HEADER_SEEK)?;
        let id: u16 = self.rng.next_u16();
        Ok(Query {
            writer,
            header: Header::new_query(id),
        })
    }

    /// Send a DNS A record query.
    ///
    /// This will only send a DNS query, it will not wait for a reply from the
    /// DNS server.
    ///
    /// The return `u16` is the transaction ID, use that get the response with
    /// [`response`](Self::response).
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::OutOfMemory`]
    pub fn a_question<W5500: Udp>(
        &mut self,
        w5500: &mut W5500,
        hostname: &Hostname,
    ) -> Result<u16, Error<W5500::Error>> {
        self.query(w5500)?.question(Qtype::A, hostname)?.send()
    }

    /// Retrieve a DNS response after sending an [`a_question`]
    ///
    /// # Arguments
    ///
    /// * `w5500`: The W5500 device that implements the [`Udp`] trait.
    /// * `buf`: A buffer for reading the hostname.
    ///   Hostnames can be up to 255 bytes.
    /// * `id`: The DNS query ID as provided by `query`.
    ///
    /// # Errors
    ///
    /// This method can only return:
    ///
    /// * [`Error::Other`]
    /// * [`Error::UnexpectedEof`]
    /// * [`Error::WouldBlock`]
    ///
    /// [`a_question`]: Self::a_question
    pub fn response<'a, W5500: Udp>(
        &self,
        w5500: &'a mut W5500,
        buf: &'a mut [u8],
        id: u16,
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

        if header.id() != id {
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
            rr_idx: 0,
        })
    }
}
