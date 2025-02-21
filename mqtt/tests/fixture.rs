use mqttbytes::{
    Protocol::V5,
    QoS,
    v5::{
        ConnAck, Connect, ConnectProperties, ConnectReturnCode, Packet, Publish, RetainForwardRule,
        SubAck as MbSubAck, Subscribe, SubscribeFilter, SubscribeReasonCode,
    },
};
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::{Duration, Instant},
};
use w5500_hl::TcpReader;
use w5500_mqtt::{
    Client, ClientId, Error, Event, SRC_PORT, SubAck, SubAckReasonCode,
    ll::{
        Registers, Sn,
        net::{Ipv4Addr, SocketAddrV4},
    },
};
use w5500_regsim::W5500;

pub struct Server {
    listener: TcpListener,
    stream: Option<TcpStream>,
}

impl Server {
    pub fn new(server_port: u16) -> Self {
        Self {
            listener: TcpListener::bind(format!("127.0.0.1:{server_port}")).expect("bind failed"),
            stream: None,
        }
    }

    pub fn accept(&mut self) {
        let (stream, _addr) = self.listener.accept().expect("accept failed");
        stream.set_nonblocking(true).unwrap();
        self.stream.replace(stream);
    }

    pub fn read(&mut self) -> Result<Packet, mqttbytes::Error> {
        let mut buf = bytes::BytesMut::with_capacity(u16::MAX as usize);
        buf.resize(u16::MAX as usize, 0);
        let n: usize = self
            .stream
            .as_ref()
            .unwrap()
            .read(&mut buf)
            .expect("read failed");
        mqttbytes::v5::read(&mut buf, n)
    }

    pub fn send_connack(&mut self) {
        self.send_connack_code(ConnectReturnCode::Success)
    }

    pub fn write_all(&mut self, buf: &[u8]) {
        let mut stream: &TcpStream = self.stream.as_ref().unwrap();
        stream.write_all(buf).unwrap();
        stream.flush().unwrap()
    }

    pub fn send_connack_code(&mut self, code: ConnectReturnCode) {
        let conn_ack: ConnAck = ConnAck {
            session_present: false,
            code,
            properties: None,
        };
        let mut buf = bytes::BytesMut::new();
        conn_ack.write(&mut buf).unwrap();
        self.write_all(&buf)
    }

    pub fn send_suback(&mut self, pkid: u16) {
        let sub_ack: MbSubAck = MbSubAck {
            pkid,
            return_codes: vec![SubscribeReasonCode::QoS0],
            properties: None,
        };
        let mut buf = bytes::BytesMut::new();
        sub_ack.write(&mut buf).unwrap();
        self.write_all(&buf)
    }

    pub fn publish(&mut self, topic: &str, payload: &[u8]) {
        let publish: Publish = Publish::new(topic, QoS::AtMostOnce, payload);
        let mut buf = bytes::BytesMut::new();
        publish.write(&mut buf).unwrap();
        self.write_all(&buf)
    }
}

pub struct Monotonic {
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

const CLIENT_ID_STR: &str = "test";
pub const CLIENT_ID: ClientId<'static> = ClientId::new_unwrapped(CLIENT_ID_STR);

pub struct Fixture {
    pub mono: Monotonic,
    pub server: Server,
    pub client: Client<'static>,
    pub w5500: W5500,
}

impl Fixture {
    pub fn new(server_port: u16) -> Self {
        stderrlog::new()
            .verbosity(4)
            .timestamp(stderrlog::Timestamp::Nanosecond)
            .init()
            .ok();

        let server: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::LOCALHOST, server_port);
        let mut client: Client = Client::new(Sn::Sn0, SRC_PORT, server);
        client.set_client_id(CLIENT_ID);

        let mut w5500: W5500 = W5500::default();
        w5500.set_socket_buffer_logging(false);

        Self {
            mono: Default::default(),
            server: Server::new(server_port),
            client,
            w5500,
        }
    }

    pub fn with_client(client: Client<'static>, server_port: u16) -> Self {
        let mut ret = Self::new(server_port);
        ret.client = client;
        ret
    }

    pub fn connect(&mut self) {
        assert!(matches!(
            self.client_process().unwrap(),
            Event::CallAfter(10)
        ));
        self.server.accept();
        assert!(matches!(
            self.client_process().unwrap(),
            Event::CallAfter(10)
        ));
        self.server_expect(Packet::Connect(Connect {
            protocol: V5,
            keep_alive: 900,
            client_id: CLIENT_ID_STR.to_string(),
            clean_session: true,
            last_will: None,
            login: None,
            properties: Some(ConnectProperties {
                session_expiry_interval: None,
                receive_maximum: None,
                max_packet_size: Some(2048),
                topic_alias_max: None,
                request_response_info: None,
                request_problem_info: None,
                user_properties: vec![],
                authentication_method: None,
                authentication_data: None,
            }),
        }));
        self.server.send_connack();
        assert!(matches!(self.client_process().unwrap(), Event::ConnAck));
    }

    pub fn server_expect(&mut self, packet: Packet) {
        let actual = self.server.read().unwrap();
        assert_eq!(actual, packet);
    }

    #[allow(clippy::type_complexity)]
    pub fn client_process(
        &mut self,
    ) -> Result<
        Event<<W5500 as Registers>::Error, TcpReader<W5500>>,
        Error<<W5500 as Registers>::Error>,
    > {
        self.client
            .process(&mut self.w5500, self.mono.monotonic_secs())
    }

    pub fn subscribe(&mut self, topic: &str) {
        let pkt_id: u16 = self.client.subscribe(&mut self.w5500, topic).unwrap();
        assert_ne!(pkt_id, 0);

        let mut expected_filter = SubscribeFilter::new(topic.to_string(), QoS::AtMostOnce);
        expected_filter.set_nolocal(true);
        expected_filter.set_retain_forward_rule(RetainForwardRule::Never);

        self.server_expect(Packet::Subscribe(Subscribe {
            pkid: pkt_id,
            filters: vec![expected_filter],
            properties: None,
        }));

        self.server.send_suback(pkt_id);

        match self.client_process().unwrap() {
            Event::SubAck(ack)
                if ack
                    == SubAck {
                        pkt_id,
                        code: SubAckReasonCode::QoS0,
                    } => {}
            x => panic!("Expected SubAck, got {x:?}"),
        }
    }

    pub fn client_expect_publish(&mut self, topic: &str, payload: &[u8]) {
        const TIMEOUT: Duration = Duration::from_secs(1);
        let start: Instant = Instant::now();

        let mut reader = loop {
            let event = self
                .client
                .process(&mut self.w5500, self.mono.monotonic_secs())
                .unwrap();

            match event {
                Event::Publish(reader) => break reader,
                e => log::info!("Unexpected event {e:?}"),
            }

            let elapsed: Duration = start.elapsed();
            if elapsed > TIMEOUT {
                panic!("Expected Publish event got nothing after {elapsed:?}");
            }
        };

        let mut topic_buf: Vec<u8> = vec![0; topic.len()];
        let mut payload_buf: Vec<u8> = vec![0; payload.len()];

        let n: u16 = reader.read_topic(&mut topic_buf).unwrap();
        assert_eq!(usize::from(n), topic.len());

        let n: u16 = reader.read_payload(&mut payload_buf).unwrap();
        assert_eq!(usize::from(n), payload.len());

        assert_eq!(topic.as_bytes(), topic_buf);
        assert_eq!(payload, payload_buf);

        assert_eq!(usize::from(reader.topic_len()), topic.len());
        assert_eq!(usize::from(reader.payload_len()), payload.len());

        reader.done().unwrap();
    }
}
