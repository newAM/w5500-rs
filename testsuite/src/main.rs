use ftdi_embedded_hal::{
    embedded_hal::digital::v2::OutputPin as EhOutputPin, Delay, FtHal, InputPin, OutputPin, Spi,
};
use libftd2xx::Ft232h;
use rand_core::{OsRng, RngCore};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};
use w5500_dhcp::{Client as DhcpClient, Hostname};
use w5500_dns::Client as DnsClient;
use w5500_ll::{
    blocking::vdm::W5500,
    net::{Eui48Addr, Ipv4Addr, SocketAddrV4},
    reset, Registers, Sn, VERSION,
};
use w5500_mqtt::{Client as MqttClient, ClientId, Event, SRC_PORT as MQTT_SRC_PORT};

// change this for your network
const MQTT_SERVER: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 4), 1883);

// locally administered MAC address
const MAC_ADDRESS: Eui48Addr = Eui48Addr::new(0x82, 0x33, 0x56, 0x78, 0x9A, 0xBC);
const DHCP_SN: Sn = Sn::Sn0;
const MQTT_SN: Sn = Sn::Sn1;
const DNS_SN: Sn = Sn::Sn2;
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

struct TestArgs<'a> {
    w5500: &'a mut W5500<Spi<'a, Ft232h>, OutputPin<'a, Ft232h>>,
    mono: &'a Monotonic,
    dns: Option<Ipv4Addr>,
}

#[allow(clippy::type_complexity)]
const TESTS: [(fn(&mut TestArgs), &str); 3] = [
    (dhcp_bind, "dhcp_bind"),
    (mqtt_connect, "mqtt_connect"),
    (dns_query, "dns_query"),
];

fn dhcp_bind(ta: &mut TestArgs) {
    let dhcp_seed: u64 = (&mut OsRng).next_u64();
    let mut dhcp_client: DhcpClient = DhcpClient::new(DHCP_SN, dhcp_seed, MAC_ADDRESS, HOSTNAME);
    dhcp_client.setup_socket(ta.w5500).unwrap();
    let start: Instant = Instant::now();
    loop {
        dhcp_client
            .process(ta.w5500, ta.mono.monotonic_secs())
            .unwrap();
        if dhcp_client.is_bound() {
            log::info!("DHCP is bound");
            break;
        }
        let sn_ir = ta.w5500.sn_ir(DHCP_SN).unwrap();
        if sn_ir.any_raised() {
            log::info!("sn_ir={sn_ir:?}");
        }
        let elapsed = Instant::now().duration_since(start);
        if elapsed > Duration::from_secs(6) {
            panic!("DHCP failed to bind in {elapsed:?}");
        }
        // not required, makes looking at my logic analyzer easier
        sleep(Duration::from_millis(30));
    }

    ta.dns = dhcp_client.dns();
}

fn mqtt_connect(ta: &mut TestArgs) {
    log::info!("Connecting to MQTT server at {MQTT_SERVER}");
    let mut mqtt_client: MqttClient = MqttClient::new(MQTT_SN, MQTT_SRC_PORT, MQTT_SERVER);
    mqtt_client.set_client_id(CLIENT_ID);
    let start: Instant = Instant::now();
    while !matches!(
        mqtt_client
            .process(ta.w5500, ta.mono.monotonic_secs())
            .unwrap(),
        Event::None
    ) {
        let elapsed = Instant::now().duration_since(start);
        if elapsed > Duration::from_secs(3) {
            panic!("DHCP failed to bind in {elapsed:?}");
        }
    }
}

fn dns_query(ta: &mut TestArgs) {
    let dns_seed: u64 = (&mut OsRng).next_u64();
    let mut dns_client: DnsClient = DnsClient::new(DNS_SN, 16385, ta.dns.unwrap(), dns_seed);

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
    while let Some(answer) = response.next_answer().unwrap() {
        println!("name={:?}", answer.name);
        println!("qtype={:?}", answer.qtype);
        println!("class={:?}", answer.class);
        println!("ttl={:?}", answer.ttl);
        println!("rdata={:?}", answer.rdata);
    }
}

fn main() {
    // setup logging
    stderrlog::new()
        .verbosity(3)
        .timestamp(stderrlog::Timestamp::Nanosecond)
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

    let mono: Monotonic = Monotonic::default();
    let mut args: TestArgs = TestArgs {
        w5500: &mut w5500,
        mono: &mono,
        dns: None,
    };

    for (idx, (f, name)) in TESTS.iter().enumerate() {
        println!("[{}/{}] Running test {}", idx + 1, TESTS.len(), name);
        f(&mut args);
    }
}
