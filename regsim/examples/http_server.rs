//! This is a simulation of the W5500 registers that runs on your local machine.
//!
//! This does not require any embedded hardware to use.
//!
//! This code is very minimal to make this example readable.
//! Do as I say, not as I do: Hard coded DNS is bad.
//!
//! **Note:** This will communicate external network services.

use std::{thread::sleep, time::Duration};

use w5500_hl::{Common, Tcp};
use w5500_ll::{Registers, Sn, VERSION};
use w5500_regsim::W5500;

// Socket to use for HTTP, this could be any of them
const HTTP_SOCKET: Sn = Sn::Sn5;
// Port to server HTTP on
const HTTP_PORT: u16 = 80;

const RESPONSE200: &[u8] =
    b"HTTP/1.1 200 OK\r\nServer: W5500\r\nContent-Type: text/html\r\n\r\n<h1>Hello World!</h1>";

const RESPONSE400: &[u8] =
    b"HTTP/1.1 400 Bad Request\r\nServer: W5500\r\nContent-Type: text/html\r\n\r\n<h1>Bad Request</h1>";

// This is a "large" buffer because HTTP is a large protocol.
// Global mutable buffers are unsafe, but you probably do not want to put a
// 2048B buffer on the stack for an embedded system.
static mut BUF: [u8; 2048] = [0; 2048];

fn main() {
    // this enables the logging built into the register simulator
    stderrlog::new()
        .verbosity(4)
        .timestamp(stderrlog::Timestamp::Nanosecond)
        .init()
        .unwrap();

    let mut w5500: W5500 = W5500::default();
    assert_eq!(w5500.version().unwrap(), VERSION); // sanity check

    // in a real embedded system there is a lot more boilerplate such as:
    // - DHCP (or setting a static IP)
    // - setting a valid EUI-48 MAC address
    // - Checking link up at the physical layer
    //
    // the register simulation allows us to cheat a little since your PC
    // (hopefully) already has a valid IP/MAC/Gateway/subnet mask

    // start serving
    w5500
        .tcp_listen(HTTP_SOCKET, HTTP_PORT)
        .expect("Failed to open a listener");

    // TODO: check that the socket was successful before printing this
    println!("Serving HTML on 127.0.0.1:{}", HTTP_PORT);

    // wait for the RECV interrupt, indicating there is data to read
    loop {
        let sn_ir = w5500.sn_ir(HTTP_SOCKET).unwrap();
        if sn_ir.recv_raised() {
            w5500.set_sn_ir(HTTP_SOCKET, sn_ir).unwrap();
            break;
        }
        if sn_ir.discon_raised() | sn_ir.timeout_raised() {
            panic!("Socket disconnected while waiting for RECV");
        }
        sleep(Duration::from_millis(250));
    }

    // Read the HTTP request from the client
    // Safety: buf is only borrowed mutably in one location
    let rx_bytes: usize = w5500.tcp_read(HTTP_SOCKET, unsafe { &mut BUF }).unwrap();
    // Truncate the buffer to the number of bytes read
    // Safety: BUF is only borrowed mutably in one location
    let filled_buf: &[u8] = unsafe { &BUF[..rx_bytes] };

    // httparse is avaliable for embedded systems
    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut req = httparse::Request::new(&mut headers);
    match req.parse(filled_buf) {
        Ok(httparse::Status::Complete(_)) => {
            // respond to GET requests with a hello world page
            let response: &[u8] = if req.method == Some("GET") {
                RESPONSE200
            } else {
                RESPONSE400
            };
            println!("Sending HTTP response");
            w5500
                .tcp_write(HTTP_SOCKET, response)
                .expect("Failed to send HTTP response");
        }
        // handle partial data by waiting for more data from the client
        Ok(httparse::Status::Partial) => todo!("Implement handling for partial HTTP data"),
        Err(e) => {
            println!("Error parsing HTTP data, closing socket: {:?}", e);
            w5500.close(HTTP_SOCKET).expect("Failed to close socket");
        }
    }
}
