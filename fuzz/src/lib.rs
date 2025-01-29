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

impl W5500<'_> {
    pub fn set_socket_status(&mut self, socket_status: SocketStatus) {
        self.socket_status = socket_status
    }
}

impl Registers for W5500<'_> {
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
            SocketCommand::Disconnect => self.socket_status = SocketStatus::Closed,
            _ => panic!("Unexpected socket command {cmd:?}"),
        }

        Ok(())
    }

    #[inline]
    fn sn_sr(&mut self, sn: Sn) -> Result<Result<SocketStatus, u8>, Self::Error> {
        self.ptr += 1;
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

#[derive(Default)]
pub struct NotRng {
    val: u8,
}

impl NotRng {
    #[inline]
    fn next_byte(&mut self) -> u8 {
        self.val = self.val.wrapping_add(1);
        self.val
    }
}

impl rand_core::RngCore for NotRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.next_byte().into()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.next_byte().into()
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.iter_mut().for_each(|b| *b = self.next_byte());
    }
}

impl rand_core::CryptoRng for NotRng {}
