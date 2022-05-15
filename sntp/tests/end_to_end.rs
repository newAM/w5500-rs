use std::net::UdpSocket;
use w5500_regsim::W5500;
use w5500_sntp::{
    ll::{
        net::{Ipv4Addr, SocketAddrV4},
        Sn,
    },
    Client, LeapIndicator, Stratum,
};

struct Server {
    socket: UdpSocket,
    client_port: u16,
}

impl Server {
    pub fn new(addr: SocketAddrV4, client_port: u16) -> Self {
        let addr: String = format!("{}", addr);
        Self {
            socket: UdpSocket::bind(addr).expect("Unable to bind UDP socket"),
            client_port,
        }
    }

    pub fn recv(&mut self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0; 2048];
        let n: usize = self
            .socket
            .recv(&mut buf)
            .expect("Failed to read from server socket");
        buf.truncate(n);
        buf
    }

    pub fn send(&mut self, msg: &[u8]) {
        let addr: String = format!("127.0.0.1:{}", self.client_port);
        self.socket
            .send_to(msg, addr)
            .expect("Failed to send from server socket");
    }
}

#[test]
fn end_to_end() {
    stderrlog::new()
        .verbosity(4)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();

    const SERVER_PORT: u16 = 12345;
    const CLIENT_PORT: u16 = SERVER_PORT + 1;
    const SERVER_ADDR: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), SERVER_PORT);

    let client = Client::new(Sn::Sn0, CLIENT_PORT, Ipv4Addr::LOCALHOST);
    let mut server = Server::new(SERVER_ADDR, CLIENT_PORT);

    let mut w5500: W5500 = W5500::default();
    w5500.set_socket_buffer_logging(false);
    client.request(&mut w5500, None).unwrap();

    let buf: Vec<u8> = server.recv();
    assert_eq!(buf.len(), 48);
    let mut buf_iter = buf.iter();
    assert_eq!(*buf_iter.next().unwrap(), 0x23);
    buf_iter.for_each(|byte| assert_eq!(*byte, 0));

    // pulled from a real-world SNTP server reply, 216.239.35.4
    const REPLY: [u8; 48] = [
        0x24, 0x01, 0x00, 0xec, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x47, 0x4f, 0x4f,
        0x47, 0xe5, 0xfd, 0x82, 0x24, 0x23, 0xec, 0x4b, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0xe5, 0xfd, 0x82, 0x24, 0x23, 0xec, 0x4b, 0x13, 0xe5, 0xfd, 0x82, 0x24, 0x23,
        0xec, 0x4b, 0x15,
    ];

    server.send(&REPLY);

    let reply = client.on_recv_interrupt(&mut w5500).unwrap();

    assert_eq!(reply.leap_indicator, LeapIndicator::NoWarning);
    assert_eq!(reply.stratum, Stratum::Primary);
    assert_eq!(reply.precision, 0xEC_u8 as i8);
    assert_eq!(reply.root_delay.to_bits(), 0);
    assert_eq!(reply.root_dispersion.to_bits(), 0x09);
    assert_eq!(reply.reference_identifier, *b"GOOG");
    assert_eq!(
        reply.reference_timestamp.to_bits(),
        0xe5_fd_82_24_23_ec_4b_12
    );
    assert_eq!(reply.originate_timestamp.to_bits(), 0);
    assert_eq!(reply.receive_timestamp.to_bits(), 0xe5_fd_82_24_23_ec_4b_13);
    assert_eq!(
        reply.transmit_timestamp.to_bits(),
        0xe5_fd_82_24_23_ec_4b_15
    );
}
