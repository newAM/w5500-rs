//! Helpers for fuzzing W5500 crates.
use w5500_ll::{Protocol, Registers, Sn, SocketCommand, SocketMode, SocketStatus};

pub const FUZZ_SN: Sn = Sn::Sn0;

/// Mock W5500 for fuzzing.
pub struct W5500<'a> {
    fuzz: &'a [u8],
    ptr: usize,
    socket_status: SocketStatus,
    mode: SocketMode,
}

impl<'a> From<&'a [u8]> for W5500<'a> {
    #[inline]
    fn from(fuzz: &'a [u8]) -> Self {
        Self {
            fuzz,
            ptr: 0,
            socket_status: SocketStatus::Closed,
            mode: SocketMode::DEFAULT,
        }
    }
}

impl<'a> W5500<'a> {
    pub fn set_socket_status(&mut self, socket_status: SocketStatus) {
        self.socket_status = socket_status
    }
}

impl<'a> Registers for W5500<'a> {
    type Error = ();

    #[inline]
    fn read(&mut self, _addr: u16, _block: u8, data: &mut [u8]) -> Result<(), Self::Error> {
        let window = self.fuzz.get(self.ptr..(self.ptr + data.len())).ok_or(())?;
        data.copy_from_slice(window);
        self.ptr += data.len();
        Ok(())
    }

    #[inline]
    fn write(&mut self, _addr: u16, _block: u8, _data: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    #[inline]
    fn set_sn_cr(&mut self, _sn: Sn, cmd: SocketCommand) -> Result<(), Self::Error> {
        match cmd {
            SocketCommand::Open => match self.mode.protocol().unwrap() {
                Protocol::Tcp => self.socket_status = SocketStatus::Init,
                Protocol::Udp => self.socket_status = SocketStatus::Udp,
                mode => panic!("unexpected socket mode for open {mode:?}"),
            },
            SocketCommand::Connect => self.socket_status = SocketStatus::Established,
            SocketCommand::Close => self.socket_status = SocketStatus::Closed,
            SocketCommand::Recv | SocketCommand::Send => (),
            _ => panic!("Unexpected socket command {cmd:?}"),
        }

        Ok(())
    }

    #[inline]
    fn sn_sr(&mut self, sn: Sn) -> Result<Result<SocketStatus, u8>, Self::Error> {
        if sn != Sn::Sn0 {
            Ok(Ok(SocketStatus::Closed))
        } else {
            Ok(Ok(self.socket_status))
        }
    }

    #[inline]
    fn set_sn_mr(&mut self, _sn: Sn, mode: SocketMode) -> Result<(), Self::Error> {
        self.mode = mode;
        Ok(())
    }
}
