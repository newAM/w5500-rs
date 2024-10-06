#![allow(dead_code)]

mod fixture;
use fixture::Fixture;
use mqttbytes::{
    v5::{Connect, ConnectProperties, ConnectReturnCode, Packet},
    Protocol::V5,
};
use w5500_mqtt::{
    ll::{
        net::{Ipv4Addr, SocketAddrV4},
        Sn::Sn0,
    },
    Client, ConnectReasonCode, Error, Event, SRC_PORT,
};

#[test]
fn connect_no_client_id() {
    const PORT: u16 = 12345;
    let client: Client = Client::new(Sn0, SRC_PORT, SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT));
    let mut fixture = Fixture::with_client(client, PORT);
    assert!(matches!(
        fixture.client_process().unwrap(),
        Event::CallAfter(10)
    ));
    fixture.server.accept();
    assert!(matches!(
        fixture.client_process().unwrap(),
        Event::CallAfter(10)
    ));
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

#[test]
fn connect_with_login() {
    const PORT: u16 = 12345;
    let mut client: Client =
        Client::new(Sn0, SRC_PORT, SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT));
    client.set_credentials("mqtt-user", "password");

    let mut fixture = Fixture::with_client(client, PORT);
    assert!(matches!(
        fixture.client_process().unwrap(),
        Event::CallAfter(10)
    ));
    fixture.server.accept();
    assert!(matches!(
        fixture.client_process().unwrap(),
        Event::CallAfter(10)
    ));
    fixture.server_expect(Packet::Connect(Connect {
        protocol: V5,
        keep_alive: 900,
        client_id: "".to_string(),
        clean_session: true,
        last_will: None,
        login: Some(mqttbytes::v5::Login {
            username: "mqtt-user".to_string(),
            password: "password".to_string(),
        }),
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

#[test]
fn connect_fail() {
    const PORT: u16 = 12344;
    let client: Client = Client::new(Sn0, SRC_PORT, SocketAddrV4::new(Ipv4Addr::LOCALHOST, PORT));
    let mut fixture = Fixture::with_client(client, PORT);
    assert!(matches!(
        fixture.client_process().unwrap(),
        Event::CallAfter(10)
    ));
    fixture.server.accept();
    assert!(matches!(
        fixture.client_process().unwrap(),
        Event::CallAfter(10)
    ));
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
    fixture.server.send_connack_code(ConnectReturnCode::Banned);

    match fixture.client_process().unwrap_err() {
        Error::ConnAck(rc) => assert_eq!(rc, ConnectReasonCode::Banned),
        e => panic!("unexpecte error: {e:?}"),
    }
}
