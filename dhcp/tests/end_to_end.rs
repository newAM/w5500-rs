use dhcproto::v4::{
    Decodable, Decoder, DhcpOption, Encodable, Encoder, Flags, HType, Message, MessageType, Opcode,
    OptionCode,
};
use std::{net::UdpSocket, time::Instant};
use w5500_dhcp::{ll::Sn, Client, Dhcp};
use w5500_hl::net::{Eui48Addr, Ipv4Addr};
use w5500_regsim::{Registers, W5500};

struct Server {
    pub socket: UdpSocket,
}

impl Server {
    pub fn recv(&mut self) -> Message {
        let mut buf: Vec<u8> = vec![0; 2048];
        let n: usize = self
            .socket
            .recv(&mut buf)
            .expect("Failed to read from server socket");
        buf.truncate(n);
        Message::decode(&mut Decoder::new(&buf)).expect("Failed to decode message from client")
    }

    pub fn send(&mut self, msg: Message) {
        let mut buf = Vec::with_capacity(2048);
        let mut e = Encoder::new(&mut buf);
        msg.encode(&mut e)
            .expect("Failed to encode message from DHCP server");

        self.socket
            .send_to(&buf, "127.0.0.1:2051")
            .expect("Failed to send from server socket");
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            socket: UdpSocket::bind("127.0.0.1:2050").expect("Unable to bind UDP socket"),
        }
    }
}

struct Monotonic {
    start: Instant,
}

impl Default for Monotonic {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Monotonic {
    pub fn monotonic_secs(&self) -> u32 {
        Instant::now()
            .duration_since(self.start)
            .as_secs()
            .try_into()
            .unwrap()
    }
}

#[test]
fn end_to_end() {
    stderrlog::new()
        .verbosity(4)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();

    const SEED: u64 = 0x1234; // normally random, but we want a deterministic XID
    const MAC: Eui48Addr = Eui48Addr::new(0x02, 0x34, 0x56, 0x78, 0xAB, 0xDE);
    const HOSTNAME: &str = "TESTING";
    let mac_with_hardware_type: Vec<u8> = {
        let mut buf: Vec<u8> = Vec::with_capacity(16);
        buf.push(0x01);
        buf.extend_from_slice(&MAC.octets);
        buf
    };

    let mut w5500: W5500 = W5500::default();
    w5500
        .set_sipr(&Ipv4Addr::LOCALHOST)
        .expect("Failed to set source IP");
    let mut dhcp: Dhcp = Dhcp::new(DHCP_SN, SEED, MAC, HOSTNAME);

    const DHCP_SN: Sn = Sn::Sn0;

    let mut server: Server = Server::default();
    let mut client_buf: Vec<u8> = vec![0; 2048];
    let mut client: Client<W5500> = Client::new(&mut w5500, &mut dhcp, &mut client_buf);

    let mono: Monotonic = Monotonic::default();
    client.poll(mono.monotonic_secs()).expect("poll");

    let msg: Message = server.recv();

    assert_eq!(msg.opcode(), Opcode::BootRequest);
    assert_eq!(msg.htype(), HType::Eth);
    assert_eq!(msg.hlen(), 6);
    assert_eq!(msg.hops(), 0);
    assert_eq!(msg.xid(), 0xcc6913b8);
    assert_eq!(msg.secs(), 0);
    assert_eq!(msg.flags().broadcast(), true);
    assert_eq!(msg.ciaddr(), std::net::Ipv4Addr::UNSPECIFIED);
    assert_eq!(msg.yiaddr(), std::net::Ipv4Addr::UNSPECIFIED);
    assert_eq!(msg.siaddr(), std::net::Ipv4Addr::UNSPECIFIED);
    assert_eq!(msg.giaddr(), std::net::Ipv4Addr::UNSPECIFIED);
    assert_eq!(msg.chaddr()[..6], MAC.octets);
    assert!(msg.sname().is_none());
    assert!(msg.fname().is_none());

    assert_eq!(
        msg.opts()
            .get(OptionCode::MessageType)
            .expect("MessageType is missing"),
        &DhcpOption::MessageType(MessageType::Discover)
    );
    assert_eq!(
        msg.opts()
            .get(OptionCode::ClientIdentifier)
            .expect("ClientIdentifier is missing"),
        &DhcpOption::ClientIdentifier(mac_with_hardware_type.clone())
    );
    assert_eq!(
        msg.opts()
            .get(OptionCode::Hostname)
            .expect("Hostname is missing"),
        &DhcpOption::Hostname(HOSTNAME.to_string())
    );

    const YIADDR: [u8; 4] = [1, 2, 3, 4];

    let mut offer: Message = Message::default();
    offer
        .set_opcode(Opcode::BootReply)
        .set_htype(HType::Eth)
        .set_hops(0)
        .set_xid(msg.xid())
        .set_flags(Flags::default().set_broadcast())
        .set_chaddr(&Ipv4Addr::LOCALHOST.octets)
        .set_yiaddr(YIADDR)
        .opts_mut()
        .insert(DhcpOption::MessageType(MessageType::Offer));

    server.send(offer);

    client
        .on_recv_interrupt(mono.monotonic_secs())
        .expect("on_recv_interrupt");

    let msg: Message = server.recv();

    assert_eq!(msg.opcode(), Opcode::BootRequest);
    assert_eq!(msg.htype(), HType::Eth);
    assert_eq!(msg.hlen(), 6);
    assert_eq!(msg.hops(), 0);
    assert_eq!(msg.xid(), 0x6b97902c);
    assert_eq!(msg.secs(), 0);
    assert_eq!(msg.flags().broadcast(), true);
    assert_eq!(msg.ciaddr(), std::net::Ipv4Addr::UNSPECIFIED);
    assert_eq!(msg.yiaddr(), std::net::Ipv4Addr::UNSPECIFIED);
    assert_eq!(msg.siaddr(), std::net::Ipv4Addr::UNSPECIFIED);
    assert_eq!(msg.giaddr(), std::net::Ipv4Addr::UNSPECIFIED);
    assert_eq!(msg.chaddr()[..6], MAC.octets);
    assert!(msg.sname().is_none());
    assert!(msg.fname().is_none());
    assert_eq!(
        msg.opts()
            .get(OptionCode::MessageType)
            .expect("MessageType is missing"),
        &DhcpOption::MessageType(MessageType::Request)
    );
    assert_eq!(
        msg.opts()
            .get(OptionCode::ClientIdentifier)
            .expect("ClientIdentifier is missing"),
        &DhcpOption::ClientIdentifier(mac_with_hardware_type)
    );
    assert_eq!(
        msg.opts()
            .get(OptionCode::Hostname)
            .expect("Hostname is missing"),
        &DhcpOption::Hostname(HOSTNAME.to_string())
    );
    assert_eq!(
        msg.opts()
            .get(OptionCode::ParameterRequestList)
            .expect("ParameterRequestList is missing"),
        &DhcpOption::ParameterRequestList(vec![
            OptionCode::SubnetMask,
            OptionCode::Router,
            OptionCode::DomainNameServer,
            OptionCode::Renewal,
            OptionCode::Rebinding
        ])
    );
    assert_eq!(
        msg.opts()
            .get(OptionCode::RequestedIpAddress)
            .expect("RequestedIpAddress is missing"),
        &DhcpOption::RequestedIpAddress(std::net::Ipv4Addr::from(YIADDR))
    );

    const SUBNET_MASK: [u8; 4] = [12, 34, 56, 78];
    const ROUTER: [u8; 4] = [11, 12, 13, 14];
    const DNS: [u8; 4] = [21, 22, 23, 24];

    let mut offer: Message = Message::default();
    offer
        .set_opcode(Opcode::BootReply)
        .set_htype(HType::Eth)
        .set_hops(0)
        .set_xid(msg.xid())
        .set_flags(Flags::default().set_broadcast())
        .set_chaddr(&Ipv4Addr::LOCALHOST.octets)
        .set_yiaddr([1, 2, 3, 4])
        .opts_mut()
        .insert(DhcpOption::MessageType(MessageType::Ack));

    offer
        .opts_mut()
        .insert(DhcpOption::SubnetMask(std::net::Ipv4Addr::from(
            SUBNET_MASK,
        )));
    offer
        .opts_mut()
        .insert(DhcpOption::ServerIdentifier(std::net::Ipv4Addr::from(
            ROUTER,
        )));
    offer
        .opts_mut()
        .insert(DhcpOption::Router(vec![ROUTER.into()]));
    offer
        .opts_mut()
        .insert(DhcpOption::DomainNameServer(vec![DNS.into()]));
    offer.opts_mut().insert(DhcpOption::AddressLeaseTime(444));
    offer.opts_mut().insert(DhcpOption::Renewal(555));
    offer.opts_mut().insert(DhcpOption::Rebinding(666));

    server.send(offer);

    client
        .on_recv_interrupt(mono.monotonic_secs())
        .expect("on_recv_interrupt");

    assert_eq!(w5500.sipr().unwrap(), Ipv4Addr::from(YIADDR));
    assert_eq!(w5500.gar().unwrap(), Ipv4Addr::from(ROUTER));
    assert_eq!(w5500.subr().unwrap(), Ipv4Addr::from(SUBNET_MASK));
}
