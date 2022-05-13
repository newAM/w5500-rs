use core::convert::Infallible;
use w5500_dns::{
    hl::{
        ll::{
            net::{Ipv4Addr, SocketAddrV4},
            Registers, Sn, SocketStatus,
        },
        UdpHeader,
    },
    Client, Qclass, Qtype, ResourceRecord, DST_PORT,
};

const SRC_PORT: u16 = 12345;
const SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DST_PORT);
const REALLY_BAD_SEED: u64 = 0;
const CLIENT: Client = Client::new(Sn::Sn0, SRC_PORT, *SERVER.ip(), REALLY_BAD_SEED);

struct MockW5500 {
    data: &'static [u8],
    header: UdpHeader,
    ptr: usize,
}

impl MockW5500 {
    pub fn new(data: &'static [u8]) -> Self {
        Self {
            data,
            header: UdpHeader {
                origin: SERVER,
                len: data.len() as u16,
            },
            ptr: 0,
        }
    }
}

impl Registers for MockW5500 {
    type Error = Infallible;

    fn read(&mut self, addr: u16, block: u8, _data: &mut [u8]) -> Result<(), Self::Error> {
        unimplemented!("read addr={:04X} block={:02X}", addr, block);
    }

    fn write(&mut self, addr: u16, block: u8, _data: &[u8]) -> Result<(), Self::Error> {
        unimplemented!("write addr={:04X} block={:02X}", addr, block);
    }

    fn sn_sr(&mut self, _: Sn) -> Result<Result<SocketStatus, u8>, Self::Error> {
        Ok(Ok(SocketStatus::Udp))
    }

    fn sn_rx_rsr(&mut self, _: Sn) -> Result<u16, Self::Error> {
        const W5500_UDP_HEADER_LEN: u16 = 8;
        Ok(W5500_UDP_HEADER_LEN + (self.data.len() as u16))
    }

    fn sn_rx_rd(&mut self, _: Sn) -> Result<u16, Self::Error> {
        Ok(0)
    }

    fn sn_rx_buf(&mut self, _: Sn, ptr: u16, buf: &mut [u8]) -> Result<(), Self::Error> {
        const UDP_HEADER_LEN: usize = 8;
        if ptr == 0 {
            // reading the UDP header
            assert_eq!(buf.len(), UDP_HEADER_LEN);

            buf[..4].copy_from_slice(&self.header.origin.ip().octets);
            buf[4..6].copy_from_slice(&self.header.origin.port().to_be_bytes());
            buf[6..8].copy_from_slice(&self.header.len.to_be_bytes());
        } else {
            assert!(usize::from(ptr) >= UDP_HEADER_LEN);
            let data_ptr: usize = usize::from(ptr) - UDP_HEADER_LEN;
            buf.copy_from_slice(&self.data[data_ptr..(data_ptr + buf.len())]);
        }
        self.ptr += buf.len();

        Ok(())
    }
}

/// Label compression at the start of the label
#[test]
fn label_compression_start() {
    const RESPONSE: [u8; 89] = [
        0x47, 0x5F, 0x81, 0x80, 0x00, 0x01, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x04, 0x64, 0x6F,
        0x63, 0x73, 0x02, 0x72, 0x73, 0x00, 0x00, 0x01, 0x00, 0x01, 0xC0, 0x0C, 0x00, 0x01, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x3C, 0x00, 0x04, 0x12, 0x41, 0xE5, 0x73, 0xC0, 0x0C, 0x00, 0x01,
        0x00, 0x01, 0x00, 0x00, 0x00, 0x3C, 0x00, 0x04, 0x12, 0x41, 0xE5, 0x50, 0xC0, 0x0C, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3C, 0x00, 0x04, 0x12, 0x41, 0xE5, 0x4C, 0xC0, 0x0C,
        0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x3C, 0x00, 0x04, 0x12, 0x41, 0xE5, 0x69,
    ];
    let mut w5500: MockW5500 = MockW5500::new(&RESPONSE);
    let mut buf: [u8; 16] = [0; 16];
    let mut response = CLIENT.response(&mut w5500, &mut buf, 0x475F).unwrap();

    assert_eq!(
        response.next_rr().unwrap(),
        Some(ResourceRecord {
            name: Some("docs.rs"),
            qtype: Ok(Qtype::A),
            class: Ok(Qclass::IN),
            ttl: 60,
            rdata: Some(Ipv4Addr::new(18, 65, 229, 115)),
        })
    );
    assert_eq!(
        response.next_rr().unwrap(),
        Some(ResourceRecord {
            name: Some("docs.rs"),
            qtype: Ok(Qtype::A),
            class: Ok(Qclass::IN),
            ttl: 60,
            rdata: Some(Ipv4Addr::new(18, 65, 229, 80)),
        })
    );
    assert_eq!(
        response.next_rr().unwrap(),
        Some(ResourceRecord {
            name: Some("docs.rs"),
            qtype: Ok(Qtype::A),
            class: Ok(Qclass::IN),
            ttl: 60,
            rdata: Some(Ipv4Addr::new(18, 65, 229, 76)),
        })
    );
    assert_eq!(
        response.next_rr().unwrap(),
        Some(ResourceRecord {
            name: Some("docs.rs"),
            qtype: Ok(Qtype::A),
            class: Ok(Qclass::IN),
            ttl: 60,
            rdata: Some(Ipv4Addr::new(18, 65, 229, 105)),
        })
    );
    assert_eq!(response.next_rr().unwrap(), None);
}

/// Label compression in the middle of the label
#[test]
fn label_compression_mid() {
    const RESPONSE: [u8; 49] = [
        0xb4, 0x20, 0x84, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x04, 0x69, 0x6d,
        0x61, 0x63, 0x05, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x00, 0x00, 0x01, 0x80, 0x01, 0x04, 0x69,
        0x4d, 0x61, 0x63, 0xc0, 0x11, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x04,
        0xc0, 0xa8, 0x00, 0x02,
    ];
    let mut w5500: MockW5500 = MockW5500::new(&RESPONSE);
    let mut buf: [u8; 1024] = [0; 1024];
    let mut response = CLIENT.response(&mut w5500, &mut buf, 0xB420).unwrap();

    assert_eq!(
        response.next_rr().unwrap(),
        Some(ResourceRecord {
            name: Some("iMac.local"),
            qtype: Ok(Qtype::A),
            class: Ok(Qclass::IN),
            ttl: 10,
            rdata: Some(Ipv4Addr::new(192, 168, 0, 2)),
        })
    );
    assert_eq!(response.next_rr().unwrap(), None);
}

/// Label compression in the middle of the label
#[test]
fn ptr_response() {
    stderrlog::new()
        .verbosity(3)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();
    const RESPONSE: [u8; 309] = [
        0x00, 0x00, 0x84, 0x00, 0x00, 0x01, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x05, 0x5f, 0x68,
        0x74, 0x74, 0x70, 0x04, 0x5f, 0x74, 0x63, 0x70, 0x05, 0x6c, 0x6f, 0x63, 0x61, 0x6c, 0x00,
        0x00, 0x0c, 0x00, 0x01, 0xc0, 0x0c, 0x00, 0x0c, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x00,
        0x09, 0x06, 0x43, 0x6c, 0x6f, 0x73, 0x65, 0x74, 0xc0, 0x0c, 0xc0, 0x2e, 0x00, 0x10, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x0a, 0x00, 0xab, 0x0f, 0x76, 0x65, 0x6e, 0x64, 0x6f, 0x72, 0x3d,
        0x53, 0x79, 0x6e, 0x6f, 0x6c, 0x6f, 0x67, 0x79, 0x0c, 0x6d, 0x6f, 0x64, 0x65, 0x6c, 0x3d,
        0x44, 0x53, 0x32, 0x31, 0x38, 0x2b, 0x14, 0x73, 0x65, 0x72, 0x69, 0x61, 0x6c, 0x3d, 0x31,
        0x39, 0x32, 0x30, 0x50, 0x43, 0x4e, 0x38, 0x34, 0x33, 0x34, 0x30, 0x32, 0x0f, 0x76, 0x65,
        0x72, 0x73, 0x69, 0x6f, 0x6e, 0x5f, 0x6d, 0x61, 0x6a, 0x6f, 0x72, 0x3d, 0x36, 0x0f, 0x76,
        0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x5f, 0x6d, 0x69, 0x6e, 0x6f, 0x72, 0x3d, 0x32, 0x13,
        0x76, 0x65, 0x72, 0x73, 0x69, 0x6f, 0x6e, 0x5f, 0x62, 0x75, 0x69, 0x6c, 0x64, 0x3d, 0x32,
        0x35, 0x35, 0x35, 0x36, 0x0f, 0x61, 0x64, 0x6d, 0x69, 0x6e, 0x5f, 0x70, 0x6f, 0x72, 0x74,
        0x3d, 0x35, 0x30, 0x30, 0x30, 0x16, 0x73, 0x65, 0x63, 0x75, 0x72, 0x65, 0x5f, 0x61, 0x64,
        0x6d, 0x69, 0x6e, 0x5f, 0x70, 0x6f, 0x72, 0x74, 0x3d, 0x35, 0x30, 0x30, 0x31, 0x1d, 0x6d,
        0x61, 0x63, 0x5f, 0x61, 0x64, 0x64, 0x72, 0x65, 0x73, 0x73, 0x3d, 0x30, 0x30, 0x3a, 0x31,
        0x31, 0x3a, 0x33, 0x32, 0x3a, 0x61, 0x37, 0x3a, 0x32, 0x66, 0x3a, 0x38, 0x65, 0xc0, 0x2e,
        0x00, 0x21, 0x00, 0x01, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x13,
        0x88, 0x06, 0x43, 0x6c, 0x6f, 0x73, 0x65, 0x74, 0xc0, 0x17, 0xc1, 0x00, 0x00, 0x1c, 0x00,
        0x01, 0x00, 0x00, 0x00, 0x0a, 0x00, 0x10, 0xfe, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02, 0x11, 0x32, 0xff, 0xfe, 0xa7, 0x2f, 0x8e, 0xc1, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00,
        0x00, 0x00, 0x0a, 0x00, 0x04, 0xc0, 0xa8, 0x01, 0x8a,
    ];
    let mut w5500: MockW5500 = MockW5500::new(&RESPONSE);
    let mut buf: [u8; 1024] = [0; 1024];
    let mut response = CLIENT.response(&mut w5500, &mut buf, 0).expect("response");
    assert_eq!(
        response.next_rr().expect("_http"),
        Some(ResourceRecord {
            name: Some("_http._tcp.local"),
            qtype: Ok(Qtype::PTR),
            class: Ok(Qclass::IN),
            ttl: 10,
            rdata: None,
        })
    );
    assert_eq!(
        response.next_rr().expect("Closet"),
        Some(ResourceRecord {
            name: Some("Closet._http._tcp.local"),
            qtype: Ok(Qtype::TXT),
            class: Ok(Qclass::IN),
            ttl: 10,
            rdata: None,
        })
    );
    assert_eq!(
        response.next_rr().expect("Closet"),
        Some(ResourceRecord {
            name: Some("Closet._http._tcp.local"),
            qtype: Ok(Qtype::SVR),
            class: Ok(Qclass::IN),
            ttl: 10,
            rdata: None,
        })
    );
    assert_eq!(
        response.next_rr().expect("Closet"),
        Some(ResourceRecord {
            name: Some("Closet.local"),
            qtype: Ok(Qtype::AAAA),
            class: Ok(Qclass::IN),
            ttl: 10,
            rdata: None,
        })
    );
    assert_eq!(
        response.next_rr().expect("Closet"),
        Some(ResourceRecord {
            name: Some("Closet.local"),
            qtype: Ok(Qtype::A),
            class: Ok(Qclass::IN),
            ttl: 10,
            rdata: Some(Ipv4Addr::new(192, 168, 1, 138)),
        })
    );
    assert_eq!(response.next_rr().unwrap(), None);
}
