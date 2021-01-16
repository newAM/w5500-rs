use w5500_ll::{
    spi::{vdm_header, AccessMode},
    Socket,
};

macro_rules! vdm_header_tests {
    ($($name:ident: $value:expr,)*) => {
    $(
        #[test]
        fn $name() {
            let ((address, block, mode), expected) = $value;
            assert_eq!(vdm_header(address, block, mode), expected);
        }
    )*
    }
}

vdm_header_tests! {
    vdm_header_0: ((0, 0, AccessMode::Read), [0, 0, 0]),
    vdm_header_1: ((0x1234, 0, AccessMode::Read), [0x12, 0x34, 0]),
    vdm_header_2: ((0, Socket::Socket0.block(), AccessMode::Read), [0, 0, 8]),
    vdm_header_3: ((0, Socket::Socket7.tx_block(), AccessMode::Read), [0, 0, 0b11110 << 3]),
    vdm_header_4: ((0, Socket::Socket7.rx_block(), AccessMode::Read), [0, 0, 0b11111 << 3]),
    vdm_header_5: ((0, 0, AccessMode::Write), [0, 0, 4]),
}
