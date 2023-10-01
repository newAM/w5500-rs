use dhcproto::v4::{
    Decodable, Decoder, DhcpOption, Encodable, Encoder, Flags, HType, Message, MessageType, Opcode,
    OptionCode,
};
use std::net::UdpSocket;
use w5500_dhcp::{
    ll::{
        net::{Eui48Addr, Ipv4Addr},
        Sn,
    },
    Client, Hostname,
};
use w5500_hl::net::SocketAddrV4;
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
        let socket = UdpSocket::bind("127.0.0.1:2050").expect("Unable to bind UDP socket");
        socket
            .set_nonblocking(true)
            .expect("failed to set socket to non-blocking");
        Self { socket }
    }
}

#[derive(Default)]
struct MockMonotonic {
    counter: u32,
}

impl MockMonotonic {
    pub fn monotonic_secs(&mut self) -> u32 {
        self.counter += 1;
        self.counter
    }
}

const HOSTNAME_STR: &str = "TESTING";
const HOSTNAME: Hostname = Hostname::new_unwrapped(HOSTNAME_STR);
const MAC: Eui48Addr = Eui48Addr::new(0x02, 0x34, 0x56, 0x78, 0xAB, 0xDE);
const YIADDR: [u8; 4] = [1, 2, 3, 4];
const SUBNET_MASK: [u8; 4] = [12, 34, 56, 78];
const ROUTER: [u8; 4] = [11, 12, 13, 14];
const DNS_1: [u8; 4] = [21, 22, 23, 24];
const DNS_2: [u8; 4] = [12, 22, 32, 42];
const NTP: [u8; 4] = [31, 32, 33, 34];

fn check_recv_request(msg: &Message, xid: u32, mac_with_hardware_type: Vec<u8>) {
    assert_eq!(msg.opcode(), Opcode::BootRequest);
    assert_eq!(msg.htype(), HType::Eth);
    assert_eq!(msg.hlen(), 6);
    assert_eq!(msg.hops(), 0);
    assert_eq!(msg.xid(), xid);
    assert_eq!(msg.secs(), 0);
    assert!(msg.flags().broadcast());
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
        &DhcpOption::Hostname(HOSTNAME_STR.to_string())
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
            OptionCode::Rebinding,
            OptionCode::NtpServers,
        ])
    );
    assert_eq!(
        msg.opts()
            .get(OptionCode::RequestedIpAddress)
            .expect("RequestedIpAddress is missing"),
        &DhcpOption::RequestedIpAddress(std::net::Ipv4Addr::from(YIADDR))
    );
}

#[test]
fn end_to_end() {
    stderrlog::new()
        .verbosity(3)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();

    // normally random, but we want a deterministic XID for testing
    const SEED: u64 = 0x1234;

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
    let mut dhcp: Client = Client::new(DHCP_SN, SEED, MAC, HOSTNAME);
    dhcp.set_broadcast_addr(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 2050));
    dhcp.set_src_port(2051);
    dhcp.set_timeout_secs(11);
    dhcp.setup_socket(&mut w5500).unwrap();

    const DHCP_SN: Sn = Sn::Sn0;

    let mut server: Server = Server::default();

    let mut mono: MockMonotonic = MockMonotonic::default();
    let next_call: u32 = dhcp.process(&mut w5500, mono.monotonic_secs()).unwrap();
    assert_eq!(next_call, 11);

    let msg: Message = server.recv();

    assert_eq!(msg.opcode(), Opcode::BootRequest);
    assert_eq!(msg.htype(), HType::Eth);
    assert_eq!(msg.hlen(), 6);
    assert_eq!(msg.hops(), 0);
    assert_eq!(msg.xid(), 0xcc6913b8);
    assert_eq!(msg.secs(), 0);
    assert!(msg.flags().broadcast());
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
        &DhcpOption::Hostname(HOSTNAME_STR.to_string())
    );

    let mut offer: Message = Message::default();
    offer
        .set_opcode(Opcode::BootReply)
        .set_htype(HType::Eth)
        .set_hops(0)
        .set_xid(msg.xid())
        .set_flags(Flags::default().set_broadcast())
        .set_chaddr(&Ipv4Addr::LOCALHOST.octets())
        .set_yiaddr(YIADDR)
        .opts_mut()
        .insert(DhcpOption::MessageType(MessageType::Offer));

    server.send(offer);

    let next_call: u32 = dhcp.process(&mut w5500, mono.monotonic_secs()).unwrap();
    assert_eq!(next_call, 11);

    let msg: Message = server.recv();
    check_recv_request(&msg, 0x6b97902c, mac_with_hardware_type.clone());

    let mut offer: Message = Message::default();
    offer
        .set_opcode(Opcode::BootReply)
        .set_htype(HType::Eth)
        .set_hops(0)
        .set_xid(msg.xid())
        .set_flags(Flags::default().set_broadcast())
        .set_chaddr(&Ipv4Addr::LOCALHOST.octets())
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
    offer.opts_mut().insert(DhcpOption::DomainNameServer(vec![
        DNS_1.into(),
        DNS_2.into(),
    ]));
    offer
        .opts_mut()
        .insert(DhcpOption::NtpServers(vec![NTP.into()]));
    const LEASE_TIME: u32 = 666;
    const T2: u32 = 555;
    const T1: u32 = 444;
    offer
        .opts_mut()
        .insert(DhcpOption::AddressLeaseTime(LEASE_TIME));
    offer.opts_mut().insert(DhcpOption::Renewal(T1));
    offer.opts_mut().insert(DhcpOption::Rebinding(T2));

    server.send(offer);

    let next_call: u32 = dhcp.process(&mut w5500, mono.monotonic_secs()).unwrap();
    const T1_NEXT_CALL: u32 = T1.saturating_sub(T1 / 8);
    assert_eq!(next_call, T1_NEXT_CALL);

    assert_eq!(w5500.sipr().unwrap(), Ipv4Addr::from(YIADDR));
    assert_eq!(w5500.gar().unwrap(), Ipv4Addr::from(ROUTER));
    assert_eq!(w5500.subr().unwrap(), Ipv4Addr::from(SUBNET_MASK));
    assert_eq!(dhcp.dns().unwrap(), Ipv4Addr::from(DNS_1));
    assert_eq!(dhcp.ntp().unwrap(), Ipv4Addr::from(NTP));

    // force T1 expiry
    let next_call: u32 = dhcp
        .process(
            &mut w5500,
            mono.monotonic_secs().saturating_add(T1_NEXT_CALL),
        )
        .unwrap();
    let msg: Message = server.recv();
    check_recv_request(&msg, 0x6d279eac, mac_with_hardware_type.clone());
    const T2_NEXT_CALL: u32 = T2
        .saturating_sub(T2 / 8)
        .saturating_sub(T1_NEXT_CALL)
        .saturating_sub(1);
    assert_eq!(next_call, T2_NEXT_CALL);

    // force t2 expiry
    let next_call: u32 = dhcp
        .process(
            &mut w5500,
            mono.monotonic_secs()
                .saturating_add(T1_NEXT_CALL)
                .saturating_add(T2_NEXT_CALL),
        )
        .unwrap();
    let msg: Message = server.recv();
    check_recv_request(&msg, 0x8809cefa, mac_with_hardware_type);
    const LEASE_NEXT_CALL: u32 = LEASE_TIME
        .saturating_sub(LEASE_TIME / 8)
        .saturating_sub(T1_NEXT_CALL)
        .saturating_sub(T2_NEXT_CALL)
        .saturating_sub(2);
    assert_eq!(next_call, LEASE_NEXT_CALL);
}
