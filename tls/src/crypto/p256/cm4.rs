use rand_core::{CryptoRng, RngCore};

#[derive(Default)]
pub struct PublicKey {
    x: [u32; 8],
    y: [u32; 8],
}

pub fn public_key_from_sec1_bytes(bytes: &[u8; 65]) -> Option<PublicKey> {
    let mut ret: PublicKey = Default::default();
    p256_cm4::octet_string_to_point(&mut ret.x, &mut ret.y, bytes).then_some(ret)
}

pub type EphemeralSecret = [u32; 8];

#[allow(unsafe_code)]
pub fn keygen<R: RngCore + CryptoRng>(rng: &mut R) -> (EphemeralSecret, [u8; 65]) {
    let mut private_key: [u32; 8] = [0; 8];
    let mut public_x: [u32; 8] = [0; 8];
    let mut public_y: [u32; 8] = [0; 8];

    loop {
        rng.fill_bytes(unsafe {
            core::mem::transmute::<&mut [u32; 8], &mut [u8; 32]>(&mut private_key)
        });
        if p256_cm4::keygen(&mut public_x, &mut public_y, &private_key) {
            break;
        }
    }

    let mut public_sec1_bytes: [u8; 65] = [0; 65];

    p256_cm4::point_to_octet_string_uncompressed(&mut public_sec1_bytes, &public_x, &public_y);

    (private_key, public_sec1_bytes)
}

pub type SharedSecret = [u8; 32];

pub fn diffie_hellman(secret: &EphemeralSecret, public: &PublicKey) -> SharedSecret {
    let mut shared: [u8; 32] = [0; 32];

    let _ignored_return_value_because_public_key_was_already_validated: bool =
        p256_cm4::ecdh_calc_shared_secret(&mut shared, secret, &public.x, &public.y);

    shared
}
