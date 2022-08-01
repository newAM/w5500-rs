use core::mem::MaybeUninit;
use rand_core::{CryptoRng, RngCore};

#[derive(Default)]
pub struct PublicKey {
    x: [u32; 8],
    y: [u32; 8],
}

pub fn public_key_from_sec1_bytes(bytes: &[u8; 65]) -> Option<PublicKey> {
    let mut ret: PublicKey = Default::default();
    unsafe {
        p256_cm4::p256_octet_string_to_point(
            ret.x.as_mut_ptr(),
            ret.y.as_mut_ptr(),
            bytes.as_ptr(),
            65,
        )
    }
    .then(|| ret)
}

pub type EphemeralSecret = [u32; 8];

pub fn keygen<R: RngCore + CryptoRng>(rng: &mut R) -> (EphemeralSecret, [u8; 65]) {
    let mut private_key: [u32; 8] = [0; 8];
    let mut public_x: [u32; 8] = [0; 8];
    let mut public_y: [u32; 8] = [0; 8];

    loop {
        rng.fill_bytes(unsafe {
            core::mem::transmute::<&mut [u32; 8], &mut [u8; 32]>(&mut private_key)
        });
        if unsafe {
            p256_cm4::p256_keygen(
                public_x.as_mut_ptr(),
                public_y.as_mut_ptr(),
                private_key.as_ptr(),
            )
        } {
            break;
        }
    }

    let mut public_sec1_bytes: MaybeUninit<[u8; 65]> = MaybeUninit::<[u8; 65]>::uninit();
    unsafe {
        p256_cm4::p256_point_to_octet_string_uncompressed(
            public_sec1_bytes.as_mut_ptr() as *mut u8,
            public_x.as_ptr(),
            public_y.as_ptr(),
        )
    };

    (private_key, unsafe { public_sec1_bytes.assume_init() })
}

pub type SharedSecret = [u8; 32];

pub fn diffie_hellman(secret: &EphemeralSecret, public: &PublicKey) -> SharedSecret {
    let mut shared: MaybeUninit<[u8; 32]> = MaybeUninit::<[u8; 32]>::uninit();

    let _ignored_return_value_because_public_key_was_already_validated: bool = unsafe {
        p256_cm4::p256_ecdh_calc_shared_secret(
            shared.as_mut_ptr() as *mut u8,
            secret.as_ptr(),
            public.x.as_ptr(),
            public.y.as_ptr(),
        )
    };

    unsafe { shared.assume_init() }
}
