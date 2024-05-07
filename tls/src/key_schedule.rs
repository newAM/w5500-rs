//! TLS key schedule.
//!
//! # References
//!
//! * [RFC 5869] HMAC-based Extract-and-Expand Key Derivation Function (HKDF)
//! * [RFC 8446 Section 7.1](https://datatracker.ietf.org/doc/html/rfc8446#section-7.1)
//!
//! [RFC 5869]: https://datatracker.ietf.org/doc/html/rfc5869

use crate::{
    crypto::p256::{EphemeralSecret, PublicKey, SharedSecret},
    AlertDescription,
};
use core::mem::size_of;
use hkdf::Hkdf;
use hmac::Mac;
use rand_core::{CryptoRng, RngCore};
use sha2::{
    digest::{
        crypto_common::generic_array::{ArrayLength, GenericArray},
        typenum::{Unsigned, U12, U32},
        OutputSizeUser,
    },
    Digest, Sha256,
};

// pre-computed SHA256 with no data
const EMPTY_HASH: [u8; 32] = [
    0xE3, 0xB0, 0xC4, 0x42, 0x98, 0xFC, 0x1C, 0x14, 0x9A, 0xFB, 0xF4, 0xC8, 0x99, 0x6F, 0xB9, 0x24,
    0x27, 0xAE, 0x41, 0xE4, 0x64, 0x9B, 0x93, 0x4C, 0xA4, 0x95, 0x99, 0x1B, 0x78, 0x52, 0xB8, 0x55,
];

const SHA256_LEN: usize = 256 / 8;
const ZEROS_OF_HASH_LEN: [u8; SHA256_LEN] = [0; SHA256_LEN];

/// Create a TLS HKDF label.
///
/// # References
///
/// * [RFC 8446 Section 7.1](https://datatracker.ietf.org/doc/html/rfc8446#section-7.1)
///
/// ```text
/// struct {
///     uint16 length = Length;
///     opaque label<7..255> = "tls13 " + Label;
///     opaque context<0..255> = Context;
/// } HkdfLabel;
/// ```
const HKDF_LABEL_LEN_MAX: usize = size_of::<u16>() + 255 + 255;
fn hkdf_label(len: u16, label: &[u8], context: &[u8]) -> heapless::Vec<u8, HKDF_LABEL_LEN_MAX> {
    let mut hkdf_label: heapless::Vec<u8, HKDF_LABEL_LEN_MAX> = heapless::Vec::new();
    hkdf_label.extend_from_slice(&len.to_be_bytes()).unwrap();

    const LABEL_PREFIX: &[u8] = b"tls13 ";
    let label_len: u8 = u8::try_from(label.len() + LABEL_PREFIX.len()).unwrap();

    hkdf_label.push(label_len).unwrap();
    hkdf_label.extend_from_slice(LABEL_PREFIX).unwrap();
    hkdf_label.extend_from_slice(label).unwrap();

    let context_len: u8 = u8::try_from(context.len()).unwrap();
    hkdf_label.push(context_len).unwrap();
    hkdf_label.extend_from_slice(context).unwrap();

    hkdf_label
}

/// TLS `HKDF-Expand-Label` function.
///
/// # References
///
/// * [RFC 8446 Section 7.1](https://datatracker.ietf.org/doc/html/rfc8446#section-7.1)
///
/// ```text
/// HKDF-Expand-Label(Secret, Label, Context, Length) =
///     HKDF-Expand(Secret, HkdfLabel, Length)
/// ```
pub(crate) fn hkdf_expand_label<N: ArrayLength<u8>>(
    secret: &Hkdf<Sha256>,
    label: &[u8],
    context: &[u8],
) -> GenericArray<u8, N> {
    let label: heapless::Vec<u8, HKDF_LABEL_LEN_MAX> = hkdf_label(N::to_u16(), label, context);
    let mut okm: GenericArray<u8, N> = Default::default();
    secret.expand(&label, &mut okm).unwrap();
    okm
}

/// TLS `Derive-Secret` function.
///
/// # References
///
/// * [RFC 8446 Section 7.1](https://datatracker.ietf.org/doc/html/rfc8446#section-7.1)
///
/// ```text
/// Derive-Secret(Secret, Label, Messages) =
///     HKDF-Expand-Label(Secret, Label,
///                       Transcript-Hash(Messages), Hash.length)
/// ```
pub(crate) fn derive_secret(
    secret: &Hkdf<Sha256>,
    label: &[u8],
    context: &[u8],
) -> GenericArray<u8, <Sha256 as OutputSizeUser>::OutputSize> {
    let label: heapless::Vec<u8, HKDF_LABEL_LEN_MAX> = hkdf_label(
        <Sha256 as OutputSizeUser>::OutputSize::to_u16(),
        label,
        context,
    );

    let mut okm: GenericArray<u8, _> = Default::default();
    secret.expand(&label, &mut okm).unwrap();
    okm
}

pub struct KeySchedule {
    client_secret: Option<EphemeralSecret>,
    server_public: Option<PublicKey>,

    // https://datatracker.ietf.org/doc/html/rfc8446#section-4.4.1
    // Many of the cryptographic computations in TLS make use of a
    // transcript hash.  This value is computed by hashing the concatenation
    // of each included handshake message, including the handshake message
    // header carrying the handshake message type and length fields, but not
    // including record layer headers.
    transcript_hash: Sha256,

    // https://datatracker.ietf.org/doc/html/rfc8446#section-5.3
    // A 64-bit sequence number is maintained separately for reading and
    // writing records.  The appropriate sequence number is incremented by
    // one after reading or writing each record.  Each sequence number is
    // set to zero at the beginning of a connection and whenever the key is
    // changed; the first record transmitted under a particular traffic key
    // MUST use sequence number 0.
    read_record_sequence_number: u64,
    write_record_sequence_number: u64,

    hkdf: Hkdf<Sha256>,
    secret: GenericArray<u8, U32>,

    client_traffic_secret: Option<Hkdf<Sha256>>,
    server_traffic_secret: Option<Hkdf<Sha256>>,
}

impl Default for KeySchedule {
    fn default() -> Self {
        let (_, hkdf): (GenericArray<u8, _>, Hkdf<Sha256>) =
            Hkdf::<Sha256>::extract(Some(&ZEROS_OF_HASH_LEN), &ZEROS_OF_HASH_LEN);
        let secret: GenericArray<u8, _> = derive_secret(&hkdf, b"derived", &EMPTY_HASH);

        Self {
            client_secret: None,
            server_public: None,
            transcript_hash: sha2::Sha256::new(),
            read_record_sequence_number: 0,
            write_record_sequence_number: 0,
            hkdf,
            secret,
            client_traffic_secret: None,
            server_traffic_secret: None,
        }
    }
}

impl KeySchedule {
    // Wrapping 2^64 - 1 is probably impossible with a W5500 running at the
    // maximum SPI bus frequency, unwrap should never occur.
    // Use `checked_add` anyway incase I did my math wrong.
    pub fn increment_read_record_sequence_number(&mut self) {
        self.read_record_sequence_number = self.read_record_sequence_number.checked_add(1).unwrap();
        trace!(
            "read_record_sequence_number={}",
            self.read_record_sequence_number
        )
    }
    pub fn increment_write_record_sequence_number(&mut self) {
        self.write_record_sequence_number =
            self.write_record_sequence_number.checked_add(1).unwrap();
        trace!(
            "write_record_sequence_number={}",
            self.write_record_sequence_number
        )
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// Create a new ephemeral client secret, and return the public key bytes
    /// as an uncompressed SEC1 encoded point.
    pub fn new_client_secret<R: RngCore + CryptoRng>(&mut self, rng: &mut R) -> [u8; 65] {
        let (private, public) = crate::crypto::p256::keygen(rng);
        self.client_secret.replace(private);
        public
    }

    pub fn update_transcript_hash(&mut self, data: &[u8]) {
        self.transcript_hash.update(data)
    }

    pub fn transcript_hash_bytes(&self) -> GenericArray<u8, U32> {
        self.transcript_hash.clone().finalize()
    }

    pub fn set_transcript_hash(&mut self, hash: Sha256) {
        self.transcript_hash = hash
    }

    pub fn transcript_hash(&self) -> Sha256 {
        self.transcript_hash.clone()
    }

    pub fn set_server_public_key(&mut self, key: PublicKey) {
        self.server_public.replace(key);
    }

    fn shared_secret(&self) -> Option<SharedSecret> {
        Some(crate::crypto::p256::diffie_hellman(
            self.client_secret.as_ref()?,
            self.server_public.as_ref()?,
        ))
    }

    fn binder_key(&mut self, psk: &[u8]) -> Hkdf<Sha256> {
        (self.secret, self.hkdf) = Hkdf::<Sha256>::extract(Some(&ZEROS_OF_HASH_LEN), psk);
        let binder_key: GenericArray<u8, U32> =
            derive_secret(&self.hkdf, b"ext binder", &EMPTY_HASH);
        Hkdf::<Sha256>::from_prk(&binder_key).unwrap()
    }

    pub fn binder(
        &mut self,
        psk: &[u8],
        truncated_transcript_hash: Sha256,
    ) -> GenericArray<u8, U32> {
        let binder_key: Hkdf<Sha256> = self.binder_key(psk);

        // The PskBinderEntry is computed in the same way as the Finished
        // message (Section 4.4.4) but with the BaseKey being the binder_key
        // derived via the key schedule from the corresponding PSK which is
        // being offered (see Section 7.1).
        //
        // finished_key = HKDF-Expand-Label(BaseKey, "finished", "", Hash.length)
        let key: GenericArray<u8, U32> = hkdf_expand_label(&binder_key, b"finished", &[]);

        let mut hmac = hmac::Hmac::<Sha256>::new_from_slice(&key).unwrap();
        hmac.update(&truncated_transcript_hash.finalize());
        hmac.finalize().into_bytes()
    }

    pub fn initialize_early_secret(&mut self) {
        let transcript_hash_bytes: GenericArray<u8, _> = self.transcript_hash_bytes();
        let client_secret: GenericArray<u8, _> =
            derive_secret(&self.hkdf, b"c e traffic", &transcript_hash_bytes);
        self.client_traffic_secret
            .replace(Hkdf::<Sha256>::from_prk(&client_secret).unwrap());

        // there is also a early_exporter_master_secret here

        self.secret = derive_secret(&self.hkdf, b"derived", &EMPTY_HASH);

        self.read_record_sequence_number = 0;
        self.write_record_sequence_number = 0;
    }

    pub fn initialize_handshake_secret(&mut self) {
        let shared_secret = self.shared_secret().unwrap();
        (self.secret, self.hkdf) = Hkdf::<Sha256>::extract(Some(&self.secret), &shared_secret);

        let transcript_hash_bytes: GenericArray<u8, _> = self.transcript_hash_bytes();
        let client_secret: GenericArray<u8, _> =
            derive_secret(&self.hkdf, b"c hs traffic", &transcript_hash_bytes);
        self.client_traffic_secret
            .replace(Hkdf::<Sha256>::from_prk(&client_secret).unwrap());

        let server_secret: GenericArray<u8, _> =
            derive_secret(&self.hkdf, b"s hs traffic", &transcript_hash_bytes);
        self.server_traffic_secret
            .replace(Hkdf::<Sha256>::from_prk(&server_secret).unwrap());

        self.secret = derive_secret(&self.hkdf, b"derived", &EMPTY_HASH);

        self.read_record_sequence_number = 0;
        self.write_record_sequence_number = 0;
    }

    pub fn initialize_master_secret(&mut self) {
        (self.secret, self.hkdf) = Hkdf::<Sha256>::extract(Some(&self.secret), &ZEROS_OF_HASH_LEN);

        let transcript_hash_bytes: GenericArray<u8, _> = self.transcript_hash_bytes();
        let client_secret: GenericArray<u8, _> =
            derive_secret(&self.hkdf, b"c ap traffic", &transcript_hash_bytes);
        self.client_traffic_secret
            .replace(Hkdf::<Sha256>::from_prk(&client_secret).unwrap());

        let server_secret: GenericArray<u8, _> =
            derive_secret(&self.hkdf, b"s ap traffic", &transcript_hash_bytes);
        self.server_traffic_secret
            .replace(Hkdf::<Sha256>::from_prk(&server_secret).unwrap());

        self.secret = derive_secret(&self.hkdf, b"derived", &EMPTY_HASH);

        self.read_record_sequence_number = 0;
        self.write_record_sequence_number = 0;
    }

    /// Update traffic secrets.
    ///
    /// # References
    ///
    /// * [RFC 8446 Section 7.2](https://datatracker.ietf.org/doc/html/rfc8446#section-7.2)
    ///
    /// ```text
    /// application_traffic_secret_N+1 =
    ///     HKDF-Expand-Label(application_traffic_secret_N,
    ///                       "traffic upd", "", Hash.length)
    /// ```
    pub fn update_traffic_secret(&mut self) {
        (self.secret, self.hkdf) = Hkdf::<Sha256>::extract(Some(&self.secret), &ZEROS_OF_HASH_LEN);

        let transcript_hash_bytes: GenericArray<u8, _> = self.transcript_hash_bytes();
        let client_secret: GenericArray<u8, _> =
            derive_secret(&self.hkdf, b"traffic upd", &transcript_hash_bytes);
        self.client_traffic_secret
            .replace(Hkdf::<Sha256>::from_prk(&client_secret).unwrap());

        let server_secret: GenericArray<u8, _> =
            derive_secret(&self.hkdf, b"traffic upd", &transcript_hash_bytes);
        self.server_traffic_secret
            .replace(Hkdf::<Sha256>::from_prk(&server_secret).unwrap());

        self.secret = derive_secret(&self.hkdf, b"derived", &EMPTY_HASH);

        self.read_record_sequence_number = 0;
        self.write_record_sequence_number = 0;
    }

    pub fn server_traffic_secret_exists(&self) -> bool {
        self.server_traffic_secret.is_some()
    }

    pub fn client_key_and_nonce(&self) -> Option<([u8; 16], [u8; 12])> {
        let traffic_secret = self.client_traffic_secret.as_ref()?;

        let key: [u8; 16] = hkdf_expand_label(traffic_secret, b"key", &[]).into();
        let mut iv: GenericArray<u8, U12> = hkdf_expand_label(traffic_secret, b"iv", &[]);
        self.write_record_sequence_number
            .to_be_bytes()
            .iter()
            .enumerate()
            .for_each(|(idx, byte)| iv[idx + 4] ^= byte);
        Some((key, iv.into()))
    }

    /// Get the server key and nonce.
    ///
    /// # References
    ///
    /// * [RFC 8446 Section 7.3](https://datatracker.ietf.org/doc/html/rfc8446#ref-sender)
    ///
    /// ```text
    /// [sender]_write_key = HKDF-Expand-Label(Secret, "key", "", key_length)
    /// ```
    pub fn server_key_and_nonce(&self) -> Option<([u8; 16], [u8; 12])> {
        let traffic_secret = self.server_traffic_secret.as_ref()?;

        let key: [u8; 16] = hkdf_expand_label(traffic_secret, b"key", &[]).into();
        let mut iv: GenericArray<u8, U12> = hkdf_expand_label(traffic_secret, b"iv", &[]);
        self.read_record_sequence_number
            .to_be_bytes()
            .iter()
            .enumerate()
            .for_each(|(idx, byte)| iv[idx + 4] ^= byte);
        Some((key, iv.into()))
    }

    /// # References
    ///
    /// * [RFC 8446 Section 4.4.4](https://datatracker.ietf.org/doc/html/rfc8446#section-4.4.4)
    ///
    /// ```text
    /// finished_key =
    ///     HKDF-Expand-Label(BaseKey, "finished", "", Hash.length)
    ///
    /// struct {
    ///     opaque verify_data[Hash.length];
    /// } Finished;
    ///
    /// verify_data =
    ///     HMAC(finished_key,
    ///          Transcript-Hash(Handshake Context,
    ///                          Certificate*, CertificateVerify*))
    /// ```
    pub fn verify_server_finished(&self, finished: &[u8; 32]) -> Result<(), AlertDescription> {
        let key: GenericArray<u8, U32> = hkdf_expand_label(
            self.server_traffic_secret.as_ref().unwrap(),
            b"finished",
            &[],
        );

        let mut hmac = hmac::Hmac::<Sha256>::new_from_slice(&key).unwrap();
        hmac.update(&self.transcript_hash_bytes());

        // Recipients of Finished messages MUST verify that the contents are
        // correct and if incorrect MUST terminate the connection with a
        // "decrypt_error" alert.
        hmac.verify_slice(finished)
            .map_err(|_| AlertDescription::DecryptError)
    }

    pub fn client_finished_verify_data(&self) -> GenericArray<u8, U32> {
        let key: GenericArray<u8, U32> = hkdf_expand_label(
            self.client_traffic_secret.as_ref().unwrap(),
            b"finished",
            &[],
        );

        let mut hmac = hmac::Hmac::<Sha256>::new_from_slice(&key).unwrap();
        hmac.update(&self.transcript_hash_bytes());
        hmac.finalize().into_bytes()
    }
}

impl ::core::fmt::Debug for KeySchedule {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> Result<(), ::core::fmt::Error> {
        write!(f, "KeySchedule {{ ... }}")
    }
}

#[cfg(feature = "defmt")]
impl ::defmt::Format for KeySchedule {
    fn format(&self, fmt: ::defmt::Formatter) {
        ::defmt::write!(fmt, "KeySchedule {{ ... }}");
    }
}
