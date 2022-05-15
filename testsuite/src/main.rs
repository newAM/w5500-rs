use ftdi_embedded_hal::{
    embedded_hal::digital::v2::OutputPin as EhOutputPin, Delay, FtHal, InputPin, OutputPin, Spi,
};
use libftd2xx::Ft232h;
use rand_core::{OsRng, RngCore};
use std::{
    process::Command,
    thread::sleep,
    time::{Duration, Instant},
};
use w5500_dhcp::{Client as DhcpClient, Hostname, State as DhcpState};
use w5500_dns::Client as DnsClient;
use w5500_hl::Tcp;
use w5500_ll::{
    blocking::vdm::W5500,
    net::{Eui48Addr, Ipv4Addr, SocketAddrV4},
    reset, Registers, Sn, VERSION,
};
use w5500_mqtt::{
    Client as MqttClient, ClientId, Error as MqttError, Event as MqttEvent,
    SRC_PORT as MQTT_SRC_PORT,
};
use w5500_sntp::{chrono, Client as SntpClient, Timestamp};
use w5500_tls::{Client as TlsClient, Event as TlsEvent};

// change this for your network
const SNTP_SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 4), 123);
const MQTT_SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 4), 1883);
const THISHOST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 3), 12345);
const HOSTNAME_THISHOST: Hostname = Hostname::new_unwrapped("openssl");

// locally administered MAC address
const MAC_ADDRESS: Eui48Addr = Eui48Addr::new(0x82, 0x33, 0x56, 0x78, 0x9A, 0xBC);
const DHCP_SN: Sn = Sn::Sn0;
const MQTT_SN: Sn = Sn::Sn1;
const DNS_SN: Sn = Sn::Sn2;
const TLS_SN: Sn = Sn::Sn3;
const SNTP_SN: Sn = Sn::Sn4;
const HOSTNAME: Hostname = Hostname::new_unwrapped("w5500-testsuite");
const CLIENT_ID: ClientId = ClientId::new_unwrapped("w5500testsuite");

pub fn new_w5500(
    ftdi: &FtHal<Ft232h>,
) -> (W5500<Spi<Ft232h>, OutputPin<Ft232h>>, InputPin<Ft232h>) {
    let int = ftdi.adi7().unwrap();
    let mut rst = ftdi.ad6().unwrap();
    let mut cs = ftdi.ad3().unwrap();
    let spi = ftdi.spi().unwrap();

    cs.set_high().unwrap();

    reset(&mut rst, &mut Delay::new()).unwrap();

    let w5500 = W5500::new(spi, cs);

    (w5500, int)
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

fn dhcp_poll_bound(ta: &mut TestArgs) {
    let start: Instant = Instant::now();
    loop {
        ta.dhcp_client
            .process(ta.w5500, ta.mono.monotonic_secs())
            .unwrap();
        if ta.dhcp_client.has_lease() {
            log::info!("DHCP has lease");
            break;
        }
        let elapsed = Instant::now().duration_since(start);
        if elapsed > Duration::from_secs(6) {
            panic!("DHCP failed to bind in {elapsed:?}");
        }
        // not required, makes looking at my logic analyzer easier
        sleep(Duration::from_millis(30));
    }
}

struct TestArgs<'a> {
    w5500: &'a mut W5500<Spi<'a, Ft232h>, OutputPin<'a, Ft232h>>,
    mono: &'a Monotonic,
    dhcp_client: DhcpClient<'static>,
    mqtt_client: MqttClient<'static>,
}

macro_rules! test {
    ($func:ident) => {
        ($func, stringify!($func))
    };
}

#[allow(clippy::type_complexity)]
const TESTS: &[(fn(&mut TestArgs), &str)] = &[
    test!(dhcp_bind),
    test!(dhcp_renew),
    test!(dhcp_rebind),
    test!(dhcp_lease_expire),
    test!(mqtt_connect),
    test!(mqtt_disconnect),
    test!(dns_query),
    test!(sntp),
    test!(tls_handshake),
];

fn dhcp_bind(ta: &mut TestArgs) {
    ta.dhcp_client.setup_socket(ta.w5500).unwrap();
    dhcp_poll_bound(ta);
}

fn dhcp_renew(ta: &mut TestArgs) {
    ta.dhcp_client
        .process(
            ta.w5500,
            ta.mono.monotonic_secs() + ta.dhcp_client.t1().unwrap() + 1,
        )
        .unwrap();
    assert_eq!(ta.dhcp_client.state(), DhcpState::Renewing);
    dhcp_poll_bound(ta);
}

fn dhcp_rebind(ta: &mut TestArgs) {
    ta.dhcp_client
        .process(
            ta.w5500,
            ta.mono.monotonic_secs() + ta.dhcp_client.t2().unwrap() + 1,
        )
        .unwrap();
    assert_eq!(ta.dhcp_client.state(), DhcpState::Rebinding);
    dhcp_poll_bound(ta);
}

fn dhcp_lease_expire(ta: &mut TestArgs) {
    ta.dhcp_client
        .process(
            ta.w5500,
            ta.mono.monotonic_secs() + ta.dhcp_client.lease_time().unwrap() + 1,
        )
        .unwrap();
    assert_eq!(ta.dhcp_client.state(), DhcpState::Selecting);
    dhcp_poll_bound(ta);
}

fn mqtt_connect(ta: &mut TestArgs) {
    log::info!("Connecting to MQTT server at {MQTT_SERVER}");
    ta.mqtt_client.set_client_id(CLIENT_ID);
    let start: Instant = Instant::now();
    while !matches!(
        ta.mqtt_client
            .process(ta.w5500, ta.mono.monotonic_secs())
            .unwrap(),
        MqttEvent::None
    ) {
        let elapsed = Instant::now().duration_since(start);
        if elapsed > Duration::from_secs(3) {
            panic!("DHCP failed to bind in {elapsed:?}");
        }
    }
}

fn mqtt_disconnect(ta: &mut TestArgs) {
    log::info!("forcing MQTT server to disconnect");
    const GARBAGE: [u8; 6] = [0xFF; 6];
    let n: u16 = ta.w5500.tcp_write(MQTT_SN, &GARBAGE).unwrap();
    assert_eq!(usize::from(n), GARBAGE.len());

    let start: Instant = Instant::now();
    loop {
        let event = ta.mqtt_client.process(ta.w5500, ta.mono.monotonic_secs());

        match event {
            Err(MqttError::Disconnect) => break,
            Ok(MqttEvent::None) => (),
            _ => panic!("unexpected event {event:?}"),
        }

        let elapsed = Instant::now().duration_since(start);
        if elapsed > Duration::from_secs(3) {
            panic!("MQTT failed to disconnect in {elapsed:?}");
        }
    }
}

fn dns_query(ta: &mut TestArgs) {
    let dns_seed: u64 = (&mut OsRng).next_u64();
    let mut dns_client: DnsClient = DnsClient::new(
        DNS_SN,
        16385,
        ta.dhcp_client
            .dns()
            .expect("DHCP server did not provide a DNS server"),
        dns_seed,
    );

    const DOCSRS: Hostname = Hostname::new_unwrapped("docs.rs");
    let id: u16 = dns_client.a_question(ta.w5500, &DOCSRS).unwrap();

    let start: Instant = Instant::now();
    while ta.w5500.sn_rx_rsr(DNS_SN).unwrap() == 0 {
        let elapsed = Instant::now().duration_since(start);
        if elapsed > Duration::from_secs(3) {
            panic!("no DNS response after {elapsed:?}");
        }
    }
    let mut buf: [u8; 256] = [0; 256];
    let mut response = dns_client.response(ta.w5500, &mut buf, id).unwrap();
    while let Some(rr) = response.next_rr().unwrap() {
        println!("name={:?}", rr.name);
        println!("qtype={:?}", rr.qtype);
        println!("class={:?}", rr.class);
        println!("ttl={:?}", rr.ttl);
        println!("rdata={:?}", rr.rdata);
    }
    response.done().unwrap();
}

fn sntp(ta: &mut TestArgs) {
    let sntp_client: SntpClient = SntpClient::new(SNTP_SN, 123, SNTP_SERVER);
    sntp_client.setup_socket(ta.w5500).unwrap();

    // possible future use
    // let transmit_timestamp: Timestamp = chrono::offset::Utc::now().naive_utc().try_into().unwrap();

    sntp_client.request(ta.w5500).unwrap();

    let start: Instant = Instant::now();
    while ta.w5500.sn_rx_rsr(SNTP_SN).unwrap() == 0 {
        let elapsed = Instant::now().duration_since(start);
        if elapsed > Duration::from_secs(3) {
            panic!("no SNTP response after {elapsed:?}");
        }
    }

    let reply: w5500_sntp::Reply = sntp_client.on_recv_interrupt(ta.w5500).unwrap();

    fn ndt(timestamp: Timestamp) -> chrono::NaiveDateTime {
        chrono::NaiveDateTime::try_from(timestamp).unwrap()
    }

    println!("root_delay={}", ndt(reply.reference_timestamp));
    println!("reference_timestamp={}", ndt(reply.reference_timestamp));
    println!("originate_timestamp={}", ndt(reply.originate_timestamp));
    println!("receive_timestamp={}", ndt(reply.receive_timestamp));
    println!("transmit_timestamp={}", ndt(reply.transmit_timestamp));
}

fn tls_handshake(ta: &mut TestArgs) {
    const PSK_IDENTITY: &str = "test";
    const PSK: [u8; 32] = [
        0x2f, 0x42, 0xac, 0xe2, 0xb6, 0xbe, 0x16, 0x81, 0xb3, 0xd2, 0xfc, 0xdd, 0x4b, 0xb5, 0x7b,
        0x4f, 0xfe, 0x34, 0x84, 0xee, 0x77, 0xfd, 0xaa, 0x8e, 0x21, 0x6e, 0x32, 0x72, 0xcd, 0x78,
        0x25, 0x9d,
    ];

    let mut buf: [u8; 2048] = [0; 2048];
    let mut tls_client: TlsClient<2048> = TlsClient::new(
        TLS_SN,
        12345,
        HOSTNAME_THISHOST,
        THISHOST,
        PSK_IDENTITY.as_bytes(),
        &PSK,
        &mut buf,
    );

    let psk_id_hex: String =
        PSK.iter()
            .fold(String::with_capacity(PSK.len() * 2), |mut hex, byte| {
                hex.push_str(format!("{byte:02X}").as_str());
                hex
            });

    let mut child = Command::new("timeout")
        .arg("15")
        .arg("openssl")
        .arg("s_server")
        .arg("-accept")
        .arg(THISHOST.to_string())
        .arg("-psk_identity")
        .arg(PSK_IDENTITY)
        .arg("-psk_hint")
        .arg(PSK_IDENTITY)
        .arg("-psk")
        .arg(psk_id_hex)
        .arg("-tls1_3")
        .arg("-ciphersuites")
        .arg("TLS_AES_128_GCM_SHA256")
        .arg("-no_ticket")
        .arg("-nocert")
        .spawn()
        .unwrap();

    // wait for openSSL to startup
    // ideally this should wait for "ACCEPT" on stdout
    sleep(Duration::from_millis(20));

    let start: Instant = Instant::now();
    loop {
        let event = tls_client.process(ta.w5500, &mut OsRng, ta.mono.monotonic_secs());

        match event {
            Ok(TlsEvent::CallAfter(_)) => (),
            Ok(TlsEvent::HandshakeFinished) => break,
            _ => {
                child.kill().unwrap();
                panic!("unexpected event {event:?}")
            }
        }

        let elapsed = Instant::now().duration_since(start);
        if elapsed > Duration::from_secs(3) {
            child.kill().unwrap();
            panic!("TLS client failed to connect in {elapsed:?}");
        }
    }

    child.kill().unwrap();
}

fn main() {
    stderrlog::new()
        .verbosity(3)
        .timestamp(stderrlog::Timestamp::Microsecond)
        .init()
        .unwrap();

    // create the W5500 structure
    let device: Ft232h = libftd2xx::Ftdi::new().unwrap().try_into().unwrap();
    let ftdi: FtHal<Ft232h> = FtHal::init_freq(device, 1_000_000).unwrap();
    let (mut w5500, _int) = new_w5500(&ftdi);

    // sanity check
    assert_eq!(w5500.version().unwrap(), VERSION);

    w5500.set_shar(&MAC_ADDRESS).unwrap();
    assert_eq!(w5500.shar().unwrap(), MAC_ADDRESS);

    // reduce log spam from polling for link up
    sleep(Duration::from_secs(2));

    let dhcp_seed: u64 = (&mut OsRng).next_u64();

    let mono: Monotonic = Monotonic::default();
    let mut args: TestArgs = TestArgs {
        w5500: &mut w5500,
        mono: &mono,
        dhcp_client: DhcpClient::new(DHCP_SN, dhcp_seed, MAC_ADDRESS, HOSTNAME),
        mqtt_client: MqttClient::new(MQTT_SN, MQTT_SRC_PORT, MQTT_SERVER),
    };

    for (idx, (f, name)) in TESTS.iter().enumerate() {
        println!("[{}/{}] Running test {}", idx + 1, TESTS.len(), name);
        f(&mut args);
    }
}
