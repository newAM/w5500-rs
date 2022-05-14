use super::cipher::Aes128;
use super::ghash::GHash;

pub struct Aes128Gcm {
    cipher: Aes128,
    ghash: GHash,

    counter: [u8; 16],
    counter0_ct: [u8; 16],

    data_len: usize,
}

const AAD_LEN: usize = crate::RecordHeader::LEN;

impl Aes128Gcm {
    pub fn new(key: &[u8; 16], nonce: &[u8; 12], aad: &[u8; AAD_LEN]) -> Self {
        let mut hash_key: [u8; 16] = [0; 16];
        let cipher: Aes128 = Aes128::new(key);
        cipher.encrypt_block_inplace(&mut hash_key);

        let ghash = {
            let mut ghash = GHash::new(&hash_key);
            let mut aad_padded = [0; 16];
            aad_padded[..aad.len()].copy_from_slice(aad);
            ghash.update(&aad_padded);
            ghash
        };

        // j0 aka counter0
        let counter: [u8; 16] = {
            let mut counter: [u8; 16] = [0; 16];
            counter[..12].copy_from_slice(nonce);
            counter[15] = 1;
            counter
        };

        let mut counter0_ct: [u8; 16] = counter;
        cipher.encrypt_block_inplace(&mut counter0_ct);

        Self {
            cipher,
            ghash,
            counter,
            counter0_ct,
            data_len: 0,
        }
    }

    fn increment_counter(&mut self) {
        self.counter[15] = self.counter[15].wrapping_add(1);
        if self.counter[15] == 0 {
            // 32KiB socket memory divided by 128 bits is 2048 blocks
            // unwrap should never occur.
            self.counter[14] = unwrap!(self.counter[14].checked_add(1));
        }
    }

    fn decrypt_block_inplace_inner(&mut self, block: &mut [u8; 16]) {
        self.increment_counter();

        let mut ek: [u8; 16] = self.counter;
        self.cipher.encrypt_block_inplace(&mut ek);

        block.iter_mut().zip(ek).for_each(|(a, b)| *a ^= b);
    }

    fn encrypt_block_inplace_inner(&mut self, block: &mut [u8; 16]) {
        self.increment_counter();
        let mut ek: [u8; 16] = self.counter;
        self.cipher.encrypt_block_inplace(&mut ek);
        block.iter_mut().zip(ek).for_each(|(a, b)| *a ^= b);
    }

    pub fn encrypt_block_inplace(&mut self, block: &mut [u8; 16]) {
        self.encrypt_block_inplace_inner(block);
        self.ghash.update(block);
        self.data_len += block.len();
    }

    pub fn encrypt_remainder_inplace(&mut self, padded_block: &mut [u8; 16], len: usize) {
        debug_assert!(len <= 16, "len should be less than 1 block not {}", len);
        self.encrypt_block_inplace_inner(padded_block);

        padded_block[len..].iter_mut().for_each(|b| *b = 0);
        self.ghash.update(padded_block);

        self.data_len += len;
    }

    pub fn decrypt_inplace(&mut self, data: &mut [u8]) {
        let mut chunks = data.chunks_exact_mut(16);

        (&mut chunks).for_each(|chunk| {
            let chunk: &mut [u8; 16] = chunk.try_into().unwrap();
            self.ghash.update(chunk);
            self.decrypt_block_inplace_inner(chunk);
        });

        let rem = chunks.into_remainder();

        if !rem.is_empty() {
            let mut padded_block: [u8; 16] = [0; 16];
            padded_block[..rem.len()].copy_from_slice(rem);
            self.ghash.update(&padded_block);

            self.decrypt_block_inplace_inner(&mut padded_block);
            rem.copy_from_slice(&padded_block[..rem.len()]);
        }

        self.data_len += data.len();
    }

    pub fn finish(mut self) -> [u8; 16] {
        const ASSOCIATED_DATA_BITS: u64 = (AAD_LEN as u64) * 8;
        let buffer_bits: u64 = (self.data_len as u64) * 8;

        let mut block: [u8; 16] = [0; 16];
        block[..8].copy_from_slice(&ASSOCIATED_DATA_BITS.to_be_bytes());
        block[8..].copy_from_slice(&buffer_bits.to_be_bytes());
        self.ghash.update(&block);

        let mut tag = self.ghash.finalize();
        tag.as_mut_slice()
            .iter_mut()
            .zip(self.counter0_ct)
            .for_each(|(a, b)| *a ^= b);

        tag
    }
}

#[cfg(test)]
mod tests {
    use super::Aes128Gcm;

    #[test]
    fn encrypt_no_data() {
        const KEY: [u8; 16] = [0; 16];
        const NONCE: [u8; 12] = [0; 12];
        const AAD: [u8; 5] = [0; 5];
        const TAG: [u8; 16] = [
            160, 56, 248, 75, 148, 220, 251, 4, 34, 185, 85, 34, 190, 206, 136, 0,
        ];

        let cipher = Aes128Gcm::new(&KEY, &NONCE, &AAD);
        let tag: [u8; 16] = cipher.finish();

        assert_eq!(tag, TAG);
    }

    #[test]
    fn encrypt_1_data_block() {
        const KEY: [u8; 16] = [0; 16];
        const NONCE: [u8; 12] = [0; 12];
        const AAD: [u8; 5] = [0; 5];
        const PT: [u8; 16] = [0; 16];
        const CT: [u8; 16] = [
            3, 136, 218, 206, 96, 182, 163, 146, 243, 40, 194, 185, 113, 178, 254, 120,
        ];
        const TAG: [u8; 16] = [
            83, 180, 67, 81, 66, 78, 216, 216, 225, 252, 47, 199, 8, 126, 112, 133,
        ];

        let mut data: [u8; 16] = PT;
        let mut cipher = Aes128Gcm::new(&KEY, &NONCE, &AAD);
        cipher.encrypt_block_inplace(&mut data);
        let tag: [u8; 16] = cipher.finish();

        assert_eq!(tag, TAG);
        assert_eq!(data, CT);
    }

    #[test]
    fn decrypt_1_data_block() {
        const KEY: [u8; 16] = [0; 16];
        const NONCE: [u8; 12] = [0; 12];
        const AAD: [u8; 5] = [0; 5];
        const PT: [u8; 16] = [0; 16];
        const CT: [u8; 16] = [
            3, 136, 218, 206, 96, 182, 163, 146, 243, 40, 194, 185, 113, 178, 254, 120,
        ];
        const TAG: [u8; 16] = [
            83, 180, 67, 81, 66, 78, 216, 216, 225, 252, 47, 199, 8, 126, 112, 133,
        ];

        let mut data: [u8; 16] = CT;
        let mut cipher = Aes128Gcm::new(&KEY, &NONCE, &AAD);
        cipher.decrypt_inplace(&mut data);
        let tag: [u8; 16] = cipher.finish();

        assert_eq!(tag, TAG);
        assert_eq!(data, PT);
    }

    #[test]
    fn decrypt_padded_data() {
        const KEY: [u8; 16] = [0; 16];
        const NONCE: [u8; 12] = [0; 12];
        const AAD: [u8; 5] = [0; 5];
        const PT: [u8; 7] = [0; 7];
        const CT: [u8; 7] = [0x03, 0x88, 0xDA, 0xCE, 0x60, 0xB6, 0xA3];
        const TAG: [u8; 16] = [
            9, 75, 180, 34, 99, 68, 253, 148, 92, 120, 157, 203, 191, 156, 155, 79,
        ];

        let mut data: [u8; 7] = CT;
        let mut cipher = Aes128Gcm::new(&KEY, &NONCE, &AAD);
        cipher.decrypt_inplace(&mut data);
        let tag: [u8; 16] = cipher.finish();

        assert_eq!(tag, TAG);
        assert_eq!(data, PT);
    }
}
