use crate::{
    data::{decode_variable_byte_integer, DeserError},
    CtrlPkt,
};

#[derive(Debug)]
pub(crate) struct FixedHeader {
    pub ctrl_pkt: CtrlPkt,
    #[allow(dead_code)]
    pub flags: u8,
    // truncated from u32 to u16
    // server should never send packets over u16 because we set the maximum size
    // after connection
    // additionally the socket buffers only have 32k physical memory
    pub remaining_len: u16,
    // length of the header in bytes
    pub len: u8,
}

impl FixedHeader {
    pub fn deser(buf: &[u8]) -> Result<Self, DeserError> {
        let byte0: u8 = *buf.first().ok_or(DeserError::Fragment)?;
        let ctrl_pkt: CtrlPkt = match byte0 >> 4 {
            x if x == (CtrlPkt::RESERVED as u8) => CtrlPkt::RESERVED,
            x if x == (CtrlPkt::CONNECT as u8) => CtrlPkt::CONNECT,
            x if x == (CtrlPkt::CONNACK as u8) => CtrlPkt::CONNACK,
            x if x == (CtrlPkt::PUBLISH as u8) => CtrlPkt::PUBLISH,
            x if x == (CtrlPkt::PUBACK as u8) => CtrlPkt::PUBACK,
            x if x == (CtrlPkt::PUBREC as u8) => CtrlPkt::PUBREC,
            x if x == (CtrlPkt::PUBREL as u8) => CtrlPkt::PUBREL,
            x if x == (CtrlPkt::PUBCOMP as u8) => CtrlPkt::PUBCOMP,
            x if x == (CtrlPkt::SUBSCRIBE as u8) => CtrlPkt::SUBSCRIBE,
            x if x == (CtrlPkt::SUBACK as u8) => CtrlPkt::SUBACK,
            x if x == (CtrlPkt::UNSUBSCRIBE as u8) => CtrlPkt::UNSUBSCRIBE,
            x if x == (CtrlPkt::UNSUBACK as u8) => CtrlPkt::UNSUBACK,
            x if x == (CtrlPkt::PINGREQ as u8) => CtrlPkt::PINGREQ,
            x if x == (CtrlPkt::PINGRESP as u8) => CtrlPkt::PINGRESP,
            x if x == (CtrlPkt::DISCONNECT as u8) => CtrlPkt::DISCONNECT,
            x if x == (CtrlPkt::AUTH as u8) => CtrlPkt::AUTH,
            _ => unreachable!(),
        };

        let (remaining_len, integer_len): (u32, u8) = decode_variable_byte_integer(&buf[1..])?;

        Ok(FixedHeader {
            ctrl_pkt,
            flags: byte0 & 0xF,
            remaining_len: remaining_len.try_into().map_err(|_| DeserError::Decode)?,
            len: integer_len + 1,
        })
    }
}
