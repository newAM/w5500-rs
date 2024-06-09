# w5500-dns

DNS client for the [Wiznet W5500] SPI internet offload chip.

## Warning

Please proceed with caution, and review the code before use in a production
environment.

This code was developed for one-off hobby projects.

## Limitations

* No DNS caching.
* Only supports A queries.
* Only supports a single outstanding query.
* Only supports a single question in a query.

## Example

```rust
use w5500_dns::{hl::block, ll::Sn, servers, Client as DnsClient, Hostname, Response};

const DNS_SOCKET: Sn = Sn::Sn3;
const DNS_SRC_PORT: u16 = 45917;

let mut dns_client: DnsClient =
    DnsClient::new(DNS_SOCKET, DNS_SRC_PORT, servers::CLOUDFLARE, random_number);
let hostname: Hostname = Hostname::new("docs.rs").expect("hostname is invalid");

let mut hostname_buffer: [u8; 16] = [0; 16];

let query_id: u16 = dns_client.a_question(&mut w5500, &hostname)?;
let mut response: Response<_> =
    block!(dns_client.response(&mut w5500, &mut hostname_buffer, query_id))?;

while let Some(rr) = response.next_rr()? {
    println!("name: {:?}", rr.name);
    println!("TTL: {}", rr.ttl);
    println!("IP: {:?}", rr.rdata);
}
response.done()?;
```

## Relevant Specifications

* [RFC 1035](https://www.rfc-editor.org/rfc/rfc1035)

## Feature Flags

All features are disabled by default.

* `eh0`: Passthrough to [`w5500-hl`].
* `eh1`: Passthrough to [`w5500-hl`].
* `defmt`: Enable logging with `defmt`. Also a passthrough to [`w5500-hl`].
* `log`: Enable logging with `log`.

[`w5500-hl`]: https://crates.io/crates/w5500-hl
[Wiznet W5500]: https://docs.wiznet.io/Product/iEthernet/W5500/overview
