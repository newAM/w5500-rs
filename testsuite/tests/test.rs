#![no_std]
#![no_main]

use core::sync::atomic::{AtomicUsize, Ordering};
use defmt_rtt as _; // global logger
use panic_probe as _;
use stm32f0xx_hal::{
    gpio::{
        self,
        gpioa::{PA4, PA5, PA6, PA7},
        AF0,
    },
    pac::{Peripherals, SPI1},
    prelude::*,
    spi::{self, Spi},
};
use testsuite_assets::{
    CHUNK_SIZE, NUM_CHUNKS, PEER_UDP_ADDR, UDP_DATA, W5500_TCP_ADDR, W5500_UDP_ADDR,
};
use testsuite_assets::{
    HTTP_SOCKET, PEER_TCP_ADDR, TCP_SOCKET, UDP_SOCKET, W5500_HTTP_PORT, W5500_TCP_PORT,
};
use w5500_hl::ll::{
    blocking::vdm::W5500, spi::MODE as W5500_MODE, LinkStatus, PhyCfg, Registers, Sn,
    SocketInterrupt,
};
use w5500_hl::{Common, Error, Tcp, Udp, UdpHeader};

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", {
    // NOTE(no-CAS) `timestamps` runs with interrupts disabled
    let n = COUNT.load(Ordering::Relaxed);
    COUNT.store(n + 1, Ordering::Relaxed);
    n
});

/// Terminates the application and makes `probe-run` exit with exit-code = 0
pub fn exit() -> ! {
    loop {
        cortex_m::asm::bkpt();
    }
}

/// Worlds worst delay function.
#[inline(always)]
pub fn nop_delay_ms(ms: usize) {
    for _ in 0..(727 * ms) {
        cortex_m::asm::nop();
    }
}

/// W5500 without template type deduction.
/// VDM = Variable data mode (with a chip select pin).
type MyW5500 = w5500_hl::ll::blocking::vdm::W5500<
    Spi<
        SPI1,
        PA5<gpio::Alternate<AF0>>,
        PA6<gpio::Alternate<AF0>>,
        PA7<gpio::Alternate<AF0>>,
        spi::EightBit,
    >,
    PA4<gpio::Output<gpio::PushPull>>,
>;

static mut BUFFER: [u8; 2048] = [0; 2048];

/// Interrupt polling.
///
/// Polling is just for testing purposes.
/// You should use replace this with an interrupt handler,
/// using the interrupt pin provided by the W5500.
fn poll_int<T: Registers<Error = E>, E: core::fmt::Debug>(w5500: &mut T, sn: Sn, interrupt: u8) {
    defmt::info!("Polling for interrupt on Socket{}", u8::from(sn));
    loop {
        let sn_ir = w5500.sn_ir(sn).unwrap();
        if u8::from(sn_ir) & interrupt != 0x00 {
            defmt::info!("Got interrupt on Socket{}", u8::from(sn));
            w5500.set_sn_ir(sn, interrupt).unwrap();
            break;
        }
        if sn_ir.discon_raised() {
            panic!("{:?} disconnected while polling", sn);
        }
        if sn_ir.timeout_raised() {
            panic!("{:?} timed out while polling", sn);
        }
        if sn_ir.any_raised() {
            panic!("{:?} unhandled IRQ {:02X}", sn, u8::from(sn_ir));
        }
    }
}

// See https://crates.io/crates/defmt-test/0.1.0 for more documentation
// (e.g. about the 'state' feature)
#[defmt_test::tests]
mod tests {
    use super::*;
    use defmt::{assert, assert_eq};

    #[init]
    fn init() -> MyW5500 {
        let mut dp = Peripherals::take().unwrap();
        let mut rcc = {
            let rcc = dp.RCC;
            rcc.apb2enr.modify(|_, w| w.syscfgen().set_bit());
            rcc.configure().sysclk(8.mhz()).freeze(&mut dp.FLASH)
        };
        let gpioa = dp.GPIOA.split(&mut rcc);
        let gpiob = dp.GPIOB.split(&mut rcc);

        let ((w5500_cs, mut w5500_rst), spi1_pins) = cortex_m::interrupt::free(move |cs| {
            gpiob.pb0.into_pull_down_input(cs); // INT
            (
                (
                    gpioa.pa4.into_push_pull_output(cs), // W5500 CS
                    gpioa.pa3.into_push_pull_output(cs), // W5500 RST
                ),
                (
                    gpioa.pa5.into_alternate_af0(cs), // W5500 SCK
                    gpioa.pa6.into_alternate_af0(cs), // W5500 MISO
                    gpioa.pa7.into_alternate_af0(cs), // W5500 MOSI
                ),
            )
        });
        let spi1 = Spi::spi1(dp.SPI1, spi1_pins, W5500_MODE, 1.mhz(), &mut rcc);

        let mut w5500 = W5500::new(spi1, w5500_cs);

        // reset the W5500
        defmt::debug!("Resetting the W5500");
        w5500_rst.set_high().unwrap();
        nop_delay_ms(1);
        w5500_rst.set_low().unwrap();
        nop_delay_ms(1);
        w5500_rst.set_high().unwrap();
        nop_delay_ms(3);

        w5500.set_shar(&testsuite_assets::MAC).unwrap();
        w5500.set_sipr(&testsuite_assets::W5500_IP).unwrap();
        w5500.set_gar(&testsuite_assets::GATEWAY).unwrap();
        w5500.set_subr(&testsuite_assets::SUBNET_MASK).unwrap();

        defmt::debug!("Polling for link up");
        let mut attempts: usize = 0;
        loop {
            let phy_cfg: PhyCfg = w5500.phycfgr().unwrap();
            if phy_cfg.lnk() == LinkStatus::Up {
                break;
            }
            assert!(attempts < 50, "Failed to link up in 5s");
            nop_delay_ms(100);
            attempts += 1;
        }

        w5500
    }

    #[test]
    fn would_block(w5500: &mut MyW5500) {
        w5500
            .udp_bind(UDP_SOCKET, testsuite_assets::W5500_UDP_PORT)
            .unwrap();

        let mut buf: [u8; 1] = [0];
        assert!(
            matches!(
                w5500.udp_peek_from(UDP_SOCKET, &mut buf).unwrap_err(),
                Error::WouldBlock
            ),
            "udp_peek_from should block"
        );
        assert!(
            matches!(
                w5500.udp_peek_from_header(UDP_SOCKET).unwrap_err(),
                Error::WouldBlock
            ),
            "udp_peek_from_header should block"
        );
        assert!(
            matches!(
                w5500.udp_recv_from(UDP_SOCKET, &mut buf).unwrap_err(),
                Error::WouldBlock
            ),
            "udp_recv_from should block"
        );

        w5500.close(UDP_SOCKET).unwrap();
    }

    #[test]
    fn tcp_server(w5500: &mut MyW5500) {
        w5500.tcp_listen(HTTP_SOCKET, W5500_HTTP_PORT).unwrap();

        poll_int(w5500, HTTP_SOCKET, SocketInterrupt::RECV_MASK);

        // safety: BUFFER is only borrowed mutably once
        let bytes_read: usize = w5500.tcp_read(HTTP_SOCKET, unsafe { &mut BUFFER }).unwrap();
        let filled_buffer: &[u8] = unsafe { &BUFFER[..bytes_read] };

        // request method
        assert_eq!(filled_buffer[..4], [b'G', b'E', b'T', b' ']);
        // request URI
        assert_eq!(filled_buffer[4..6], [b'/', b' ']);
        // request version
        assert_eq!(
            filled_buffer[6..14],
            [b'H', b'T', b'T', b'P', b'/', b'1', b'.', b'1']
        );

        const RESPONSE200: &[u8] =
            b"HTTP/1.1 200 OK\r\nServer: W5500\r\nContent-Type: text/plain;charset=UTF-8\r\n\r\nHello World";

        let bytes_sent: usize = w5500.tcp_write(HTTP_SOCKET, RESPONSE200).unwrap();
        assert_eq!(bytes_sent, RESPONSE200.len());

        poll_int(w5500, HTTP_SOCKET, SocketInterrupt::SENDOK_MASK);

        // disconnect from the peer
        w5500.tcp_disconnect(HTTP_SOCKET).unwrap();
    }

    #[test]
    fn tcp_client(w5500: &mut MyW5500) {
        // prepare the buffer with some data
        for idx in 0..unsafe { BUFFER }.len() {
            unsafe { BUFFER[idx] = idx as u8 };
        }

        w5500
            .tcp_connect(TCP_SOCKET, W5500_TCP_PORT, &PEER_TCP_ADDR)
            .unwrap();

        assert_eq!(w5500.local_addr(TCP_SOCKET).unwrap(), W5500_TCP_ADDR);

        poll_int(w5500, TCP_SOCKET, SocketInterrupt::CON_MASK);

        // write 32x 1234 byte chunks
        // this tests the socket pointer rollover handling in tcp_write
        for i in 0..NUM_CHUNKS {
            defmt::debug!("Chunk {}", i);

            // ensure there is space for the entire chunk
            loop {
                let fsr: u16 = w5500.sn_tx_fsr(TCP_SOCKET).unwrap();
                if usize::from(fsr) >= CHUNK_SIZE {
                    break;
                }
            }

            let n: usize = w5500
                .tcp_write(TCP_SOCKET, unsafe { &BUFFER[..CHUNK_SIZE] })
                .unwrap();
            assert_eq!(n, CHUNK_SIZE);

            poll_int(w5500, TCP_SOCKET, SocketInterrupt::SENDOK_MASK);
        }

        w5500.tcp_disconnect(TCP_SOCKET).unwrap();
    }

    #[test]
    fn udp(w5500: &mut MyW5500) {
        w5500
            .udp_bind(UDP_SOCKET, testsuite_assets::W5500_UDP_PORT)
            .unwrap();

        assert_eq!(w5500.local_addr(UDP_SOCKET).unwrap(), W5500_UDP_ADDR);

        let n: usize = w5500
            .udp_send_to(UDP_SOCKET, UDP_DATA, &testsuite_assets::PEER_UDP_ADDR)
            .unwrap();
        assert_eq!(n, UDP_DATA.len());

        poll_int(w5500, UDP_SOCKET, SocketInterrupt::SENDOK_MASK);

        // this should go to the same address
        let n: usize = w5500.udp_send(UDP_SOCKET, UDP_DATA).unwrap();
        assert_eq!(n, UDP_DATA.len());

        poll_int(w5500, UDP_SOCKET, SocketInterrupt::SENDOK_MASK);
        poll_int(w5500, UDP_SOCKET, SocketInterrupt::RECV_MASK);

        let header: UdpHeader = w5500.udp_peek_from_header(UDP_SOCKET).unwrap();
        assert_eq!(header.len as usize, UDP_DATA.len());
        assert_eq!(header.origin, PEER_UDP_ADDR);

        let mut buf: [u8; UDP_DATA.len()] = [0; UDP_DATA.len()];
        let (n, header) = w5500.udp_peek_from(UDP_SOCKET, &mut buf).unwrap();
        assert_eq!(n, UDP_DATA.len());
        assert_eq!(n, header.len as usize);
        assert_eq!(header.origin, PEER_UDP_ADDR);
        assert_eq!(buf, UDP_DATA);

        let mut buf: [u8; UDP_DATA.len()] = [0; UDP_DATA.len()];
        let (n, from) = w5500.udp_recv_from(UDP_SOCKET, &mut buf).unwrap();
        assert_eq!(n, UDP_DATA.len());
        assert_eq!(from, PEER_UDP_ADDR);
        assert_eq!(buf, UDP_DATA);

        // queue should be empty now
        assert!(
            matches!(
                w5500.udp_peek_from_header(UDP_SOCKET).unwrap_err(),
                Error::WouldBlock
            ),
            "udp_peek_from_header should block"
        );

        w5500.close(UDP_SOCKET).unwrap();
    }
}
