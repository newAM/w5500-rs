use std::convert::Infallible;
use w5500_hl::{Error, Udp};
use w5500_ll::{
    net::{Ipv4Addr, SocketAddrV4},
    Protocol, Registers, Sn, SocketCommand, SocketMode, SocketStatus,
};

/// Tests debug asserts that ensure the socket is opened as UDP.
mod socket_status_debug_assert {
    use super::*;

    struct MockRegisters {}

    impl Registers for MockRegisters {
        type Error = Infallible;

        fn sn_rx_rsr(&mut self, _socket: Sn) -> Result<u16, Self::Error> {
            Ok(1024)
        }

        fn sn_sr(&mut self, _socket: Sn) -> Result<Result<SocketStatus, u8>, Self::Error> {
            Ok(SocketStatus::try_from(u8::from(SocketStatus::Closed)))
        }

        fn read(&mut self, _address: u16, _block: u8, _data: &mut [u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn write(&mut self, _address: u16, _block: u8, _data: &[u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    #[test]
    #[should_panic]
    fn udp_recv_from() {
        let mut test = MockRegisters {};
        let mut buf: [u8; 1] = [0];
        test.udp_recv_from(Sn::Sn0, &mut buf).ok();
    }

    #[test]
    #[should_panic]
    fn udp_peek_from() {
        let mut test = MockRegisters {};
        let mut buf: [u8; 1] = [0];
        test.udp_peek_from(Sn::Sn0, &mut buf).ok();
    }

    #[test]
    #[should_panic]
    fn udp_peek_from_header() {
        let mut test = MockRegisters {};
        test.udp_peek_from_header(Sn::Sn0).ok();
    }

    #[test]
    #[should_panic]
    fn udp_send_to() {
        let mut test = MockRegisters {};
        let buf: [u8; 1] = [0];
        test.udp_send_to(Sn::Sn0, &buf, &SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))
            .ok();
    }

    #[test]
    #[should_panic]
    fn udp_send() {
        let mut test = MockRegisters {};
        let buf: [u8; 1] = [0];
        test.udp_send(Sn::Sn0, &buf).ok();
    }
}

/// Tests blocking UDP functions return nb::WouldBlock
mod udp_would_block_header {
    use super::*;

    struct MockRegisters {}

    impl Registers for MockRegisters {
        type Error = Infallible;

        fn sn_rx_rsr(&mut self, _socket: Sn) -> Result<u16, Self::Error> {
            Ok(5)
        }

        fn read(&mut self, _address: u16, _block: u8, _data: &mut [u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn write(&mut self, _address: u16, _block: u8, _data: &[u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn udp_peek_from() {
        let mut mock = MockRegisters {};
        let mut buf: [u8; 1] = [0];
        assert_eq!(
            mock.udp_peek_from(Sn::Sn0, &mut buf),
            Err(Error::WouldBlock)
        );
    }

    #[test]
    fn udp_peek_from_header() {
        let mut mock = MockRegisters {};
        assert_eq!(mock.udp_peek_from_header(Sn::Sn0), Err(Error::WouldBlock));
    }

    #[test]
    fn udp_recv_from() {
        let mut mock = MockRegisters {};
        let mut buf: [u8; 1] = [0];
        assert_eq!(
            mock.udp_recv_from(Sn::Sn0, &mut buf),
            Err(Error::WouldBlock)
        );
    }
}

/// Tests the udp_bind method
mod udp_bind {
    use super::*;

    const TEST_SOCKET: Sn = Sn::Sn7;
    const TEST_PORT: u16 = 0xABCD;

    struct MockRegisters {
        sn_sr: Vec<u8>,
        sn_cr: Vec<SocketCommand>,
    }

    impl Registers for MockRegisters {
        type Error = Infallible;

        fn set_sn_cr(&mut self, socket: Sn, cmd: SocketCommand) -> Result<(), Self::Error> {
            assert_eq!(socket, TEST_SOCKET);
            assert_eq!(cmd, self.sn_cr.pop().expect("Unexpected socket command"));
            Ok(())
        }

        fn set_sn_port(&mut self, socket: Sn, port: u16) -> Result<(), Self::Error> {
            assert_eq!(socket, TEST_SOCKET);
            assert_eq!(port, TEST_PORT);
            Ok(())
        }

        fn set_sn_mr(&mut self, socket: Sn, mode: SocketMode) -> Result<(), Self::Error> {
            assert_eq!(socket, TEST_SOCKET);
            assert_eq!(mode.protocol(), Ok(Protocol::Udp));
            Ok(())
        }

        fn sn_sr(&mut self, socket: Sn) -> Result<Result<SocketStatus, u8>, Self::Error> {
            assert_eq!(socket, TEST_SOCKET);
            Ok(SocketStatus::try_from(
                self.sn_sr.pop().expect("Unexpected socket status read"),
            ))
        }

        fn sn_port(&mut self, socket: Sn) -> Result<u16, Self::Error> {
            Ok(u16::from(u8::from(socket)))
        }

        fn read(&mut self, _address: u16, _block: u8, _data: &mut [u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn write(&mut self, _address: u16, _block: u8, _data: &[u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn udp_bind() {
        let mut mock = MockRegisters {
            sn_sr: vec![
                SocketStatus::Udp.into(),
                0xFE,
                SocketStatus::Established.into(),
                SocketStatus::Closed.into(),
                0xFF,
                SocketStatus::Init.into(),
            ],
            sn_cr: vec![SocketCommand::Open, SocketCommand::Close],
        };
        mock.udp_bind(TEST_SOCKET, TEST_PORT).unwrap();
    }
}
