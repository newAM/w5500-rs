use core::convert::Infallible;
use w5500_dns::{
    hl::{
        ll::{
            net::{Ipv4Addr, SocketAddrV4},
            Registers, Sn, SocketStatus,
        },
        UdpHeader,
    },
    Answer, Client, Qclass, Qtype, DST_PORT,
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
        response.next_answer().unwrap(),
        Some(Answer {
            name: Some("docs.rs"),
            qtype: Ok(Qtype::A),
            class: Ok(Qclass::IN),
            ttl: 60,
            rdata: Some(Ipv4Addr::new(18, 65, 229, 115)),
        })
    );
    assert_eq!(
        response.next_answer().unwrap(),
        Some(Answer {
            name: Some("docs.rs"),
            qtype: Ok(Qtype::A),
            class: Ok(Qclass::IN),
            ttl: 60,
            rdata: Some(Ipv4Addr::new(18, 65, 229, 80)),
        })
    );
    assert_eq!(
        response.next_answer().unwrap(),
        Some(Answer {
            name: Some("docs.rs"),
            qtype: Ok(Qtype::A),
            class: Ok(Qclass::IN),
            ttl: 60,
            rdata: Some(Ipv4Addr::new(18, 65, 229, 76)),
        })
    );
    assert_eq!(
        response.next_answer().unwrap(),
        Some(Answer {
            name: Some("docs.rs"),
            qtype: Ok(Qtype::A),
            class: Ok(Qclass::IN),
            ttl: 60,
            rdata: Some(Ipv4Addr::new(18, 65, 229, 105)),
        })
    );
    assert_eq!(response.next_answer().unwrap(), None);
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
    let mut buf: [u8; 16] = [0; 16];
    let mut response = CLIENT.response(&mut w5500, &mut buf, 0xB420).unwrap();

    assert_eq!(
        response.next_answer().unwrap(),
        Some(Answer {
            name: Some("iMac.local"),
            qtype: Ok(Qtype::A),
            class: Ok(Qclass::IN),
            ttl: 10,
            rdata: Some(Ipv4Addr::new(192, 168, 0, 2)),
        })
    );
    assert_eq!(response.next_answer().unwrap(), None);
}
