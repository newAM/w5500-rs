use crate::{write_variable_byte_integer, CtrlPkt};
use core::mem::size_of;
use w5500_hl::{io::Write, Error as HlError};

pub fn send_publish<E, Writer: Write<E>>(
    mut writer: Writer,
    topic: &str,
    payload: &[u8],
) -> Result<(), HlError<E>> {
    let topic_len: u16 = topic.len().try_into().unwrap_or(u16::MAX);
    let payload_len: u16 = payload.len().try_into().unwrap_or(u16::MAX);

    // length of the topic length field
    const TOPIC_LEN_LEN: u32 = size_of::<u16>() as u32;
    // length of the property length field
    const PROPERTY_LEN: u32 = size_of::<u8>() as u32;
    let remaining_len: u32 =
        TOPIC_LEN_LEN + u32::from(topic_len) + PROPERTY_LEN + u32::from(payload_len);

    writer.write_all(&[
        // control packet type
        // flags are all 0
        // dup=0, non-duplicate
        // qos=0, at most once delivery
        // retain=0, do not retain this message
        (CtrlPkt::PUBLISH as u8) << 4,
    ])?;
    write_variable_byte_integer(&mut writer, remaining_len)?;
    writer.write_all(&topic_len.to_be_bytes())?;
    writer.write_all(&topic.as_bytes()[..topic_len.into()])?;
    writer.write_all(&[0])?; // property length
    writer.write_all(&payload[..payload_len.into()])?;
    writer.send()?;
    Ok(())
}
