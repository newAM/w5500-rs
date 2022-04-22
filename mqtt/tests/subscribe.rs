#![allow(dead_code)]

mod fixture;
use fixture::Fixture;
use mqttbytes::{v5::Publish, QoS};
use w5500_mqtt::Event;

#[test]
fn subscribe() {
    let mut fixture = Fixture::new(12347);
    fixture.connect();
    fixture.subscribe("#");

    const TOPIC: &str = "testing";

    fixture.server.publish(TOPIC, &[1, 2, 3]);
    fixture.client_expect_publish(TOPIC, &[1, 2, 3]);
}

#[test]
fn subscribe_deep_queue() {
    let mut fixture = Fixture::new(12348);
    fixture.connect();
    fixture.subscribe("topic1");
    fixture.subscribe("topic2");
    fixture.subscribe("topic3");

    fixture.server.publish("topic1", b"cat");
    fixture.server.publish("topic2", b"dog");
    fixture.server.publish("topic3", b"fish");

    fixture.client_expect_publish("topic1", b"cat");
    fixture.client_expect_publish("topic2", b"dog");
    fixture.client_expect_publish("topic3", b"fish");
}

#[test]
fn subscribe_fragment() {
    const TOPIC: &str = "topic";
    const PAYLOAD: &[u8] = b"fragment";

    let mut fixture = Fixture::new(12349);
    fixture.connect();
    fixture.subscribe(TOPIC);

    let publish: Publish = Publish::new(TOPIC, QoS::AtMostOnce, PAYLOAD);
    let mut buf = bytes::BytesMut::new();
    publish.write(&mut buf).unwrap();

    for split_at in 0..buf.len() {
        let (a, b) = buf.split_at(split_at);

        fixture.server.write_all(&a);
        let result = fixture.client_process();
        assert!(
            matches!(result, Ok(Event::None)),
            "Unexpected result: {result:?}"
        );
        fixture.server.write_all(&b);
        fixture.client_expect_publish(TOPIC, b"fragment");
    }
}
