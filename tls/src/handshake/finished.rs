use super::HandshakeType;
use sha2::digest::crypto_common::{generic_array::GenericArray, typenum::U32};

/// Create a client finished handshake (i.e. not including record headers)
pub fn client_finished(verify_data: &GenericArray<u8, U32>) -> [u8; 36] {
    let mut buf: [u8; 36] = [0; 36];

    let len: [u8; 4] = u32::try_from(verify_data.len()).unwrap().to_be_bytes();
    buf[0] = HandshakeType::Finished.into();
    buf[1..4].copy_from_slice(&len[1..]);
    buf[4..].copy_from_slice(verify_data);

    buf
}
