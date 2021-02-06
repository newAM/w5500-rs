use std::convert::Infallible;
use w5500_hl::net::{Ipv4Addr, SocketAddrV4};
use w5500_hl::Common;
use w5500_ll::{Registers, Socket, SocketCommand, SOCKETS};

mod local_addr {
    use super::*;

    struct MockRegisters {}

    impl Registers for MockRegisters {
        type Error = Infallible;

        fn sipr(&mut self) -> Result<Ipv4Addr, Self::Error> {
            Ok(Ipv4Addr::LOCALHOST)
        }

        fn sn_port(&mut self, socket: Socket) -> Result<u16, Self::Error> {
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

        fn set_sn_cr(&mut self, socket: Socket, cmd: SocketCommand) -> Result<(), Self::Error> {
            assert_eq!(socket, Socket::Socket3);
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
        mock.close(Socket::Socket3).unwrap();
    }
}
