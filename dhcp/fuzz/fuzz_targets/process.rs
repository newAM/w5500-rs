#![no_main]
use libfuzzer_sys::fuzz_target;
use w5500_dhcp::{
    ll::{net::Eui48Addr, SocketStatus},
    Client, Hostname,
};
use w5500_fuzz::{FUZZ_SN, W5500};

const HOSTNAME: Hostname = Hostname::new_unwrapped("fuzz");
const MAC: Eui48Addr = Eui48Addr::new(0x02, 0x00, 0x11, 0x22, 0x33, 0x44);

fuzz_target!(|fuzz: &[u8]| {
    let mut client: Client = Client::new(FUZZ_SN, 0, MAC, HOSTNAME);
    let mut w5500: W5500 = fuzz.into();
    // faster then calling `client.setup_socket` each time
    w5500.set_socket_status(SocketStatus::Udp);

    let mut mono: u32 = 0;
    while let Ok(next_call) = client.process(&mut w5500, mono.into()) {
        mono += next_call.saturating_sub(1);
    }
});
