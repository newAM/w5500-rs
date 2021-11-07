use isahc::{
    http::{StatusCode, Version},
    ReadResponseExt,
};
use std::{
    io::Read,
    net::{TcpListener, UdpSocket},
};
use testsuite_assets::{CHUNK_SIZE, NUM_CHUNKS, W5500_IP, W5500_UDP_PORT};
fn main() {
    {
        let url = format!("http://{}", W5500_IP);
        println!("Sending HTTP GET request to: {}", url);
        let mut response = isahc::get(url).unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.version(), Version::HTTP_11);
        assert_eq!(response.headers().get("Server").unwrap(), "W5500");
        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            "text/plain;charset=UTF-8"
        );
        assert_eq!(response.text().unwrap(), "Hello World");
        println!("HTTP GET test PASSED");
    }

    {
        let local_addr = format!(
            "{}:{}",
            testsuite_assets::PEER_IP,
            testsuite_assets::PEER_TCP_PORT
        );
        println!("Listening on {}", local_addr);
        let listener = TcpListener::bind(&local_addr).unwrap();
        let (mut socket, _) = listener.accept().unwrap();

        for i in 0..NUM_CHUNKS {
            println!("Chunk {}", i);
            let mut buf = vec![0; CHUNK_SIZE];
            socket.read_exact(&mut buf).unwrap();
            for idx in 0..buf.len() {
                assert_eq!(buf[idx], idx as u8);
            }
        }
        println!("TCP client tests PASSED");
    }

    {
        let local_addr = format!(
            "{}:{}",
            testsuite_assets::PEER_IP,
            testsuite_assets::PEER_UDP_PORT
        );
        println!("Binding a UDP socket to {}", local_addr);
        let socket = UdpSocket::bind(&local_addr).unwrap();
        let mut buf = vec![0; 2048];
        for _ in 0..2usize {
            let (n, from) = socket.recv_from(&mut buf).unwrap();
            assert_eq!(&buf[..n], testsuite_assets::UDP_DATA);
            assert_eq!(from.port(), W5500_UDP_PORT);
            let ip = match from.ip() {
                std::net::IpAddr::V4(ip) => ip,
                std::net::IpAddr::V6(_) => {
                    panic!("Unexpected IPv6")
                }
            };
            assert_eq!(ip.octets(), W5500_IP.octets);
        }

        let w5500_addr = format!("{}:{}", W5500_IP, W5500_UDP_PORT);
        println!("Sending data to {}", w5500_addr);
        socket
            .send_to(testsuite_assets::UDP_DATA, &w5500_addr)
            .unwrap();
    }

    println!("Done all tests");
}
