#![no_main]
use libfuzzer_sys::fuzz_target;
use w5500_dns::{
    ll::{net::Ipv4Addr, SocketStatus},
    Client,
};
use w5500_fuzz::{FUZZ_SN, W5500};

fuzz_target!(|fuzz: &[u8]| {
    let client: Client = Client::new(FUZZ_SN, 0, Ipv4Addr::UNSPECIFIED, 0);

    let mut w5500: W5500 = fuzz.into();
    w5500.set_socket_status(SocketStatus::Udp);

    let mut buf: [u8; 256] = [0; 256];
    if let Ok(mut response) = client.response(&mut w5500, &mut buf, 0) {
        while let Ok(Some(_rr)) = response.next_rr() {}
    }
});
