#![allow(dead_code)]

mod fixture;
use fixture::{Fixture, MQTT_SERVER};
use mqttbytes::{
    v5::{Connect, ConnectProperties, Packet},
    Protocol::V5,
};
use w5500_mqtt::{ll::Sn::Sn0, Client, Event, SRC_PORT};

#[test]
fn connect_no_client_id() {
    let client: Client = Client::new(Sn0, SRC_PORT, MQTT_SERVER);
    let mut fixture = Fixture::from(client);
    assert_eq!(fixture.client_process().unwrap(), Event::CallAfter(10));
    fixture.server.accept();
    assert_eq!(fixture.client_process().unwrap(), Event::CallAfter(10));
    fixture.server_expect(Packet::Connect(Connect {
        protocol: V5,
        keep_alive: 900,
        client_id: "".to_string(),
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
}
