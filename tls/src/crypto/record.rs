use crate::{
    AlertDescription, GCM_TAG_LEN,
    crypto::Aes128Gcm,
    io::Buffer,
    record::{ContentType, RecordHeader},
};
use core::cmp::min;
use subtle::ConstantTimeEq;
use w5500_hl::ll::{Registers, Sn, SocketCommand};

pub fn decrypt_record_inplace<const N: usize, W5500: Registers>(
    w5500: &mut W5500,
    sn: Sn,
    server_key: &[u8; 16],
    server_nonce: &[u8; 12],
    header: &RecordHeader,
    buf: &mut Buffer<N>,
) -> Result<Result<ContentType, u8>, AlertDescription> {
    let mut cipher = Aes128Gcm::new(server_key, server_nonce, header.as_bytes());

    let sn_rx_rsr: u16 = w5500
        .sn_rx_rsr(sn)
        .map_err(|_| AlertDescription::InternalError)?;
    if sn_rx_rsr < header.length() {
        error!(
            "sn_rx_rsr < header.length; {} < {}",
            sn_rx_rsr,
            header.length()
        );
        return Err(AlertDescription::DecodeError);
    }
    let mut sn_rx_rd: u16 = w5500
        .sn_rx_rd(sn)
        .map_err(|_| AlertDescription::InternalError)?;

    let mut remain: u16 = header.length().saturating_sub(GCM_TAG_LEN as u16);

    if remain == 0 {
        error!("record is too short to contain ContentType");
        return Err(AlertDescription::DecodeError);
    }

    let content_type: Result<ContentType, u8> = loop {
        let mut block: [u8; 16] = [0; 16];
        let data_len: u16 = min(16, remain);

        // read ciphertext
        w5500
            .sn_rx_buf(sn, sn_rx_rd, &mut block[..data_len.into()])
            .map_err(|_| AlertDescription::InternalError)?;

        // decrypt
        cipher.decrypt_inplace(&mut block[..data_len.into()]);

        // write plaintext
        buf.extend_from_slice(&block[..data_len.into()])?;

        sn_rx_rd = sn_rx_rd.wrapping_add(data_len);
        remain -= data_len;
        if remain == 0 {
            break buf.pop_tail().unwrap().try_into();
        }
    };

    let client_tag: [u8; 16] = cipher.finish();
    let mut server_tag: [u8; 16] = [0; 16];
    w5500
        .sn_rx_buf(sn, sn_rx_rd, &mut server_tag)
        .map_err(|_| AlertDescription::InternalError)?;

    sn_rx_rd = sn_rx_rd.wrapping_add(16);
    w5500
        .set_sn_rx_rd(sn, sn_rx_rd)
        .map_err(|_| AlertDescription::InternalError)?;
    w5500
        .set_sn_rx_rd(sn, sn_rx_rd)
        .map_err(|_| AlertDescription::InternalError)?;
    w5500
        .set_sn_cr(sn, SocketCommand::Recv)
        .map_err(|_| AlertDescription::InternalError)?;

    if bool::from(client_tag.ct_eq(&server_tag)) {
        Ok(content_type)
    } else {
        Err(AlertDescription::BadRecordMac)
    }
}

// This will also send the record.
pub fn encrypt_record_inplace<W5500: Registers>(
    w5500: &mut W5500,
    sn: Sn,
    client_key: &[u8; 16],
    client_nonce: &[u8; 12],
    mut head: u16,
    tail: u16,
    content_type: ContentType,
) -> Result<(), W5500::Error> {
    const CONTENT_TYPE_LEN: u16 = 1;

    // data length without tag, header, and content type
    let data_len: u16 = tail.wrapping_sub(head);

    let header: RecordHeader = RecordHeader::ser(
        ContentType::ApplicationData,
        tail.wrapping_add(CONTENT_TYPE_LEN + (GCM_TAG_LEN as u16))
            .wrapping_sub(head),
    );

    w5500.set_sn_tx_buf(
        sn,
        head.wrapping_sub(RecordHeader::LEN as u16),
        header.as_bytes(),
    )?;

    let mut cipher = Aes128Gcm::new(client_key, client_nonce, header.as_bytes());

    for _ in 0..(data_len / 16) {
        let mut block: [u8; 16] = [0; 16];

        // read plaintext
        w5500.sn_tx_buf(sn, head, &mut block)?;

        // encrypt
        cipher.encrypt_block_inplace(&mut block);

        // write ciphertext
        w5500.set_sn_tx_buf(sn, head, &block)?;

        head = head.wrapping_add(16);
    }

    let mut remain: u16 = data_len % 16;
    let mut block: [u8; 16] = [0; 16];

    // read remaining plaintext
    w5500.sn_tx_buf(sn, head, &mut block[..remain.into()])?;

    // append content type
    block[usize::from(remain)] = content_type.into();
    remain += 1;

    // encrypt
    cipher.encrypt_remainder_inplace(&mut block, remain.into());

    // write ciphertext
    w5500.set_sn_tx_buf(sn, head, &block[..remain.into()])?;

    head = head.wrapping_add(remain);

    // write tag
    let tag: [u8; GCM_TAG_LEN] = cipher.finish();
    w5500.set_sn_tx_buf(sn, head, &tag)?;

    w5500.set_sn_tx_wr(sn, head.wrapping_add(GCM_TAG_LEN as u16))?;
    w5500.set_sn_cr(sn, SocketCommand::Send)?;

    Ok(())
}
