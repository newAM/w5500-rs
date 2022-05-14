//! This is loosely based on [`tiny-aes-c`].
//!
//! I was going to replace this with `RustCrypto/aes`, but the final binary size
//! increased significantly as a result.
//!
//! Comparing `.text` sizes in a `thumbv7em-none-eabi` end application:
//!
//! | implementation                         | `-O3`            | `-Os`            |
//! |----------------------------------------|------------------|------------------|
//! | Baseline                               | 187,280          | 147,252          |
//! | `RustCrypto/aes` with `aes_compact`    | 190,748 (+3,468) | 150,152 (+2,900) |
//! | `RustCrypto/aes` without `aes_compact` | 192,256 (+4,976) | 151,028 (+3,776) |
//!
//! ```toml
//! [profile.release]
//! codegen-units = 1
//! debug = 2
//! debug-assertions = false
//! incremental = false
//! lto = false # does nothing with codegen-units = 1
//! opt-level = # 3 or "s"
//! overflow-checks = false
//! ```
//!
//! `-O3` `nm` with `RustCrypto/aes` + `aes_compact`:
//!
//! ```text
//! 04e t aes::soft::fixslice::memshift32
//! 064 T <aes::soft::Aes128Enc as cipher::block::BlockEncrypt>::encrypt_with_backend
//! 096 t aes::soft::fixslice::mix_columns_0
//! 09a t aes::soft::fixslice::xor_columns
//! 0f8 t aes::soft::fixslice::inv_bitslice
//! 11e T aes::soft::fixslice::aes128_encrypt
//! 154 t aes::soft::fixslice::mix_columns_1
//! 168 T aes::soft::fixslice::aes128_key_schedule
//! 24a t aes::soft::fixslice::bitslice
//! 326 t aes::soft::fixslice::sub_bytes
//! ```
//!
//! `-O3` `nm` with this module:
//!
//! ```text
//! 40e t w5500_tls::crypto::aes::cipher::Aes128::encrypt_block_inplace
//! ```
//!
//! [`tiny-aes-c`]: https://github.com/kokke/tiny-AES-c

const SBOX: [u8; 256] = [
    //0     1    2      3     4    5     6     7      8    9     A      B    C     D     E     F
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

// The round constant word array, RCON[i], contains the values given by
// x to the power (i-1) being powers of x (x is denoted as {02}) in the field GF(2^8)
const RCON: [u8; 11] = [
    0x8d, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36,
];

const NB: usize = 4;
const NK: usize = 4;
const NR: usize = 10;

fn sub_bytes(block: &mut [u8; 16]) {
    block
        .iter_mut()
        .for_each(|byte| *byte = SBOX[*byte as usize]);
}

#[allow(clippy::erasing_op, clippy::identity_op)]
fn shift_rows(block: &mut [u8; 16]) {
    // Rotate first row 1 columns to left
    let mut temp: u8 = block[0 * 4 + 1];
    block[0 * 4 + 1] = block[1 * 4 + 1];
    block[1 * 4 + 1] = block[2 * 4 + 1];
    block[2 * 4 + 1] = block[3 * 4 + 1];
    block[3 * 4 + 1] = temp;

    // Rotate second row 2 columns to left
    temp = block[0 * 4 + 2];
    block[0 * 4 + 2] = block[2 * 4 + 2];
    block[2 * 4 + 2] = temp;

    temp = block[1 * 4 + 2];
    block[1 * 4 + 2] = block[3 * 4 + 2];
    block[3 * 4 + 2] = temp;

    // Rotate third row 3 columns to left
    temp = block[0 * 4 + 3];
    block[0 * 4 + 3] = block[3 * 4 + 3];
    block[3 * 4 + 3] = block[2 * 4 + 3];
    block[2 * 4 + 3] = block[1 * 4 + 3];
    block[1 * 4 + 3] = temp;
}

fn xtime(x: u8) -> u8 {
    (x << 1) ^ (((x >> 7) & 1) * 0x1b)
}

#[allow(clippy::identity_op)]
fn mix_columns(block: &mut [u8; 16]) {
    for i in 0..4 {
        let t = block[i * 4 + 0];
        let tmp = block[i * 4 + 0] ^ block[i * 4 + 1] ^ block[i * 4 + 2] ^ block[i * 4 + 3];
        let tm = block[i * 4 + 0] ^ block[i * 4 + 1];
        let tm = xtime(tm);
        block[i * 4 + 0] ^= tm ^ tmp;
        let tm = block[i * 4 + 1] ^ block[i * 4 + 2];
        let tm = xtime(tm);
        block[i * 4 + 1] ^= tm ^ tmp;
        let tm = block[i * 4 + 2] ^ block[i * 4 + 3];
        let tm = xtime(tm);
        block[i * 4 + 2] ^= tm ^ tmp;
        let tm = block[i * 4 + 3] ^ t;
        let tm = xtime(tm);
        block[i * 4 + 3] ^= tm ^ tmp;
    }
}

#[allow(clippy::identity_op)]
fn key_expansion(key: &[u8; 16]) -> [u8; 176] {
    let mut round_key: [u8; 176] = [0; 176];

    // The first round key is the key itself.
    round_key[..16].copy_from_slice(key);

    let mut tempa: [u8; 4] = [0; 4];

    // All other round keys are found from the previous round keys.
    for i in NK..(NB * (NR + 1)) {
        {
            let k: usize = (i - 1) * 4;
            tempa.copy_from_slice(&round_key[k..(k + 4)]);
        }

        if i % NK == 0 {
            // This function shifts the 4 bytes in a word to the left once.
            // [a0,a1,a2,a3] becomes [a1,a2,a3,a0]

            // Function RotWord()
            {
                let u8tmp: u8 = tempa[0];
                tempa[0] = tempa[1];
                tempa[1] = tempa[2];
                tempa[2] = tempa[3];
                tempa[3] = u8tmp;
            }

            // SubWord() is a function that takes a four-byte input word and
            // applies the S-box to each of the four bytes to produce an output word.

            // Function Subword()
            {
                tempa[0] = SBOX[tempa[0] as usize];
                tempa[1] = SBOX[tempa[1] as usize];
                tempa[2] = SBOX[tempa[2] as usize];
                tempa[3] = SBOX[tempa[3] as usize];
            }

            tempa[0] ^= RCON[i / NK];
        }

        let j: usize = i * 4;
        let k: usize = (i - NK) * 4;
        round_key[j + 0] = round_key[k + 0] ^ tempa[0];
        round_key[j + 1] = round_key[k + 1] ^ tempa[1];
        round_key[j + 2] = round_key[k + 2] ^ tempa[2];
        round_key[j + 3] = round_key[k + 3] ^ tempa[3];
    }

    round_key
}

pub struct Aes128 {
    round_key: [u8; 176],
}

impl Aes128 {
    pub fn new(key: &[u8; 16]) -> Self {
        Self {
            round_key: key_expansion(key),
        }
    }

    fn add_round_key(&self, round: usize, block: &mut [u8; 16]) {
        block
            .iter_mut()
            .enumerate()
            .for_each(|(idx, byte)| *byte ^= self.round_key[(round * NB * 4) + idx])
    }

    pub fn encrypt_block_inplace(&self, block: &mut [u8; 16]) {
        self.add_round_key(0, block);

        // There will be Nr rounds.
        // The first Nr-1 rounds are identical.
        // These Nr rounds are executed in the loop below.
        // Last one without MixColumns()
        for round in 1..=NR {
            sub_bytes(block);
            shift_rows(block);
            if round == NR {
                break;
            }
            mix_columns(block);
            self.add_round_key(round, block);
        }

        // Add round key to last round
        self.add_round_key(NR, block);
    }
}

#[cfg(test)]
mod tests {
    use super::Aes128;

    const KEY: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f,
        0x3c,
    ];
    const PT: [u8; 16] = [
        0x6b, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73, 0x93, 0x17,
        0x2a,
    ];
    const CT: [u8; 16] = [
        0x3a, 0xd7, 0x7b, 0xb4, 0x0d, 0x7a, 0x36, 0x60, 0xa8, 0x9e, 0xca, 0xf3, 0x24, 0x66, 0xef,
        0x97,
    ];

    #[test]
    fn encrypt() {
        let mut block: [u8; 16] = PT;

        let cipher = Aes128::new(&KEY);
        cipher.encrypt_block_inplace(&mut block);
        assert_eq!(block, CT);
    }
}
