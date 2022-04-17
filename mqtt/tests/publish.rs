#![allow(dead_code)]

mod fixture;
use fixture::Fixture;
use mqttbytes::v5::{Packet, Publish};

#[test]
fn publish() {
    let mut fixture = Fixture::default();
    fixture.connect();

    const TOPIC: &str = "testing";

    fixture
        .client
        .publish(&mut fixture.w5500, TOPIC, &[])
        .unwrap();

    fixture.server_expect(Packet::Publish(Publish::new(
        TOPIC,
        mqttbytes::QoS::AtMostOnce,
        vec![],
    )));
}
