#![allow(dead_code)]

mod fixture;
use fixture::Fixture;

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
