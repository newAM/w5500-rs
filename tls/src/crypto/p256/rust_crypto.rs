use super::SharedSecret;
use p256::elliptic_curve::common::Generate as _;
pub use p256::{PublicKey, ecdh::EphemeralSecret};
use rand_core::{CryptoRng, Rng};

pub fn public_key_from_sec1_bytes(bytes: &[u8; 65]) -> Option<PublicKey> {
    PublicKey::from_sec1_bytes(bytes).ok()
}

pub fn keygen<R: Rng + CryptoRng>(rng: &mut R) -> (EphemeralSecret, [u8; 65]) {
    let private_key = EphemeralSecret::generate_from_rng(rng);
    let public_sec1_bytes: [u8; 65] = p256::Sec1Point::from(private_key.public_key())
        .as_bytes()
        .try_into()
        .unwrap();
    (private_key, public_sec1_bytes)
}

pub fn diffie_hellman(secret: &EphemeralSecret, public: &PublicKey) -> SharedSecret {
    secret
        .diffie_hellman(public)
        .raw_secret_bytes()
        .as_slice()
        .try_into()
        .unwrap()
}
