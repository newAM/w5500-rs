use std::convert::Infallible;
use w5500_hl::Common;
use w5500_hl::ll::{Registers, SOCKETS, Sn, SocketCommand};
use w5500_hl::net::{Ipv4Addr, SocketAddrV4};

mod local_addr {
    use super::*;

    struct MockRegisters {}

    impl Registers for MockRegisters {
        type Error = Infallible;

        fn sipr(&mut self) -> Result<Ipv4Addr, Self::Error> {
            Ok(Ipv4Addr::LOCALHOST)
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
    fn local_addr() {
        let mut mock = MockRegisters {};
        for socket in SOCKETS.iter() {
            let expected = SocketAddrV4::new(Ipv4Addr::LOCALHOST, u16::from(u8::from(*socket)));
            assert_eq!(mock.local_addr(*socket), Ok(expected));
        }
    }
}

mod close {
    use super::*;

    struct MockRegisters {}

    impl Registers for MockRegisters {
        type Error = Infallible;

        fn set_sn_cr(&mut self, socket: Sn, cmd: SocketCommand) -> Result<(), Self::Error> {
            assert_eq!(socket, Sn::Sn3);
            assert_eq!(cmd, SocketCommand::Close);
            Ok(())
        }

        fn read(&mut self, _address: u16, _block: u8, _data: &mut [u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn write(&mut self, _address: u16, _block: u8, _data: &[u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn close() {
        let mut mock = MockRegisters {};
        mock.close(Sn::Sn3).unwrap();
    }
}

mod is_state_closed {
    use w5500_hl::ll::SocketStatus;

    use super::*;

    const SOCKET: Sn = Sn::Sn4;

    struct MockRegisters {
        states: Vec<Result<SocketStatus, u8>>,
    }

    impl Registers for MockRegisters {
        type Error = Infallible;

        fn sn_sr(&mut self, socket: Sn) -> Result<Result<SocketStatus, u8>, Self::Error> {
            assert_eq!(socket, SOCKET);
            Ok(self.states.pop().expect("Unexpected call to sn_sr"))
        }

        fn read(&mut self, _address: u16, _block: u8, _data: &mut [u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn write(&mut self, _address: u16, _block: u8, _data: &[u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn is_state_closed() {
        let mut mock = MockRegisters {
            states: vec![
                // closed state
                Ok(SocketStatus::Closed),
                // not closed states.
                Ok(SocketStatus::Udp),
                Ok(SocketStatus::Listen),
                Ok(SocketStatus::SynSent),
                Ok(SocketStatus::SynRecv),
                Ok(SocketStatus::Established),
                Ok(SocketStatus::FinWait),
                Ok(SocketStatus::Closing),
                Ok(SocketStatus::CloseWait),
                Ok(SocketStatus::TimeWait),
                Ok(SocketStatus::LastAck),
                Ok(SocketStatus::Init),
                Ok(SocketStatus::Macraw),
                Err(0x1F),
                Err(0xFF),
            ],
        };
        for _ in 0..14 {
            assert!(!mock.is_state_closed(SOCKET).unwrap());
        }
        assert!(mock.is_state_closed(SOCKET).unwrap());

        assert!(mock.states.is_empty())
    }
}

mod is_state_tcp {
    use w5500_hl::ll::SocketStatus;

    use super::*;

    const SOCKET: Sn = Sn::Sn4;

    struct MockRegisters {
        states: Vec<Result<SocketStatus, u8>>,
    }

    impl Registers for MockRegisters {
        type Error = Infallible;

        fn sn_sr(&mut self, socket: Sn) -> Result<Result<SocketStatus, u8>, Self::Error> {
            assert_eq!(socket, SOCKET);
            Ok(self.states.pop().expect("Unexpected call to sn_sr"))
        }

        fn read(&mut self, _address: u16, _block: u8, _data: &mut [u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn write(&mut self, _address: u16, _block: u8, _data: &[u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn is_state_tcp() {
        let mut mock = MockRegisters {
            states: vec![
                // TCP states
                Ok(SocketStatus::Closed),
                Ok(SocketStatus::Listen),
                Ok(SocketStatus::SynSent),
                Ok(SocketStatus::SynRecv),
                Ok(SocketStatus::Established),
                Ok(SocketStatus::FinWait),
                Ok(SocketStatus::Closing),
                Ok(SocketStatus::CloseWait),
                Ok(SocketStatus::TimeWait),
                Ok(SocketStatus::LastAck),
                // not TCP states
                Ok(SocketStatus::Init),
                Ok(SocketStatus::Udp),
                Ok(SocketStatus::Macraw),
                Err(0x1F),
                Err(0xFF),
            ],
        };
        for _ in 0..5 {
            assert!(!mock.is_state_tcp(SOCKET).unwrap());
        }
        for _ in 0..10 {
            assert!(mock.is_state_tcp(SOCKET).unwrap());
        }

        assert!(mock.states.is_empty())
    }
}

mod is_state_udp {
    use w5500_hl::ll::SocketStatus;

    use super::*;

    const SOCKET: Sn = Sn::Sn6;

    struct MockRegisters {
        states: Vec<Result<SocketStatus, u8>>,
    }

    impl Registers for MockRegisters {
        type Error = Infallible;

        fn sn_sr(&mut self, socket: Sn) -> Result<Result<SocketStatus, u8>, Self::Error> {
            assert_eq!(socket, SOCKET);
            Ok(self.states.pop().expect("Unexpected call to sn_sr"))
        }

        fn read(&mut self, _address: u16, _block: u8, _data: &mut [u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn write(&mut self, _address: u16, _block: u8, _data: &[u8]) -> Result<(), Self::Error> {
            unimplemented!()
        }
    }

    #[test]
    fn is_state_udp() {
        let mut mock = MockRegisters {
            states: vec![
                // UDP state
                Ok(SocketStatus::Udp),
                // not UDP states.
                Ok(SocketStatus::Closed),
                Ok(SocketStatus::Listen),
                Ok(SocketStatus::SynSent),
                Ok(SocketStatus::SynRecv),
                Ok(SocketStatus::Established),
                Ok(SocketStatus::FinWait),
                Ok(SocketStatus::Closing),
                Ok(SocketStatus::CloseWait),
                Ok(SocketStatus::TimeWait),
                Ok(SocketStatus::LastAck),
                Ok(SocketStatus::Init),
                Ok(SocketStatus::Macraw),
                Err(0x1F),
                Err(0xFF),
            ],
        };
        for _ in 0..14 {
            assert!(!mock.is_state_udp(SOCKET).unwrap());
        }
        assert!(mock.is_state_udp(SOCKET).unwrap());

        assert!(mock.states.is_empty())
    }
}
