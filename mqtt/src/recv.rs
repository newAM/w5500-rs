use w5500_hl::io::{Read, Seek, SeekFrom};

use crate::{
    data::DeserError, fixed_header::FixedHeader, ConnectReasonCode, CtrlPkt, Error, Event,
    PublishReader, State, StateTimeout, SubAck, SubAckReasonCode, UnSubAck, UnSubAckReasonCode,
    PROPERTY_LEN_LEN,
};

pub(crate) fn recv<E, Reader: Read<E> + Seek<E>>(
    mut reader: Reader,
    state_timeout: &mut StateTimeout,
) -> Result<Option<Event<E, Reader>>, Error<E>> {
    let mut buf: [u8; 5] = [0; 5];
    let n: u16 = reader.read(&mut buf)?;

    let header: FixedHeader = match FixedHeader::deser(&buf[..n.into()]) {
        Ok(header) => header,
        Err(DeserError::Fragment) => return Ok(None),
        Err(DeserError::Decode) => {
            error!("unable to deserialize fixed header");
            state_timeout.set_state(State::Init);
            return Err(Error::Decode);
        }
    };

    // seek to end of fixed header
    reader
        .seek(SeekFrom::Start(header.len.into()))
        .map_err(Error::map_w5500)?;

    // fragmented, can try again later
    if header.remaining_len > reader.remain() {
        return Ok(None);
    }

    debug!("recv {:?} len {}", header.ctrl_pkt, header.remaining_len);

    match header.ctrl_pkt {
        CtrlPkt::RESERVED => {
            error!("Malformed packet: control packet type is reserved");
            state_timeout.set_state(State::Init);
            Err(Error::Decode)
        }
        CtrlPkt::CONNACK => {
            if state_timeout.state != State::WaitConAck {
                error!("unexpected CONNACK in state {:?}", state_timeout.state);
                return Err(Error::Protocol);
            }
            let mut buf: [u8; 2] = [0; 2];
            reader.read_exact(&mut buf).map_err(Error::map_w5500)?;
            reader
                .seek(SeekFrom::Start(header.remaining_len.saturating_add(2)))
                .map_err(Error::map_w5500)?;
            reader.done()?;

            match ConnectReasonCode::try_from(buf[1]) {
                Err(0) => {
                    info!("Sucessfully connected");
                    state_timeout.set_state(State::Ready);
                    Ok(Some(Event::ConnAck))
                }
                Ok(code) => {
                    warn!("Unable to connect: {:?}", code);
                    state_timeout.set_state(State::Init);
                    Err(Error::ConnAck(code))
                }
                Err(e) => {
                    error!("invalid connnect reason code {:?}", e);
                    state_timeout.set_state(State::Init);
                    Err(Error::Protocol)
                }
            }
        }
        CtrlPkt::SUBACK => {
            let mut buf: [u8; 3] = [0; 3];
            let n: u16 = reader.read(&mut buf)?;
            if n != 3 {
                return Err(Error::Decode);
            }

            let (pkt_id, property_len): (&[u8], &[u8]) = buf.split_at(2);
            let pkt_id: u16 = u16::from_be_bytes(pkt_id.try_into().unwrap());
            let property_len: u8 = property_len[0];

            if property_len != 0 {
                warn!("ignoring SUBACK properties");
                reader
                    .seek(SeekFrom::Current(property_len.into()))
                    .map_err(Error::map_w5500)?;
            }

            let mut payload: [u8; 1] = [0];
            reader.read_exact(&mut payload).map_err(Error::map_w5500)?;
            let code: SubAckReasonCode = match SubAckReasonCode::try_from(payload[0]) {
                Ok(code) => code,
                Err(e) => {
                    error!("invalid SUBACK reason code value: {}", e);
                    state_timeout.set_state(State::Init);
                    return Err(Error::Protocol);
                }
            };

            reader.done()?;
            Ok(Some(Event::SubAck(SubAck { pkt_id, code })))
        }
        CtrlPkt::UNSUBACK => {
            let mut buf: [u8; 3] = [0; 3];
            let n: u16 = reader.read(&mut buf)?;
            if n != 3 {
                return Err(Error::Decode);
            }

            let (pkt_id, property_len): (&[u8], &[u8]) = buf.split_at(2);
            let pkt_id: u16 = u16::from_be_bytes(pkt_id.try_into().unwrap());
            let property_len: u8 = property_len[0];

            if property_len != 0 {
                warn!("ignoring UNSUBACK properties");
                reader
                    .seek(SeekFrom::Current(property_len.into()))
                    .map_err(Error::map_w5500)?;
            }

            let mut payload: [u8; 1] = [0];
            reader.read_exact(&mut payload).map_err(Error::map_w5500)?;
            let code: UnSubAckReasonCode = match UnSubAckReasonCode::try_from(payload[0]) {
                Ok(code) => code,
                Err(e) => {
                    error!("invalid UNSUBACK reason code value: {}", e);
                    state_timeout.set_state(State::Init);
                    return Err(Error::Protocol);
                }
            };

            reader.done()?;
            Ok(Some(Event::UnSubAck(UnSubAck { pkt_id, code })))
        }
        CtrlPkt::PUBLISH => {
            const TOPIC_LEN_LEN: u16 = 2;
            let mut topic_len: [u8; 2] = [0; 2];
            reader
                .read_exact(&mut topic_len)
                .map_err(Error::map_w5500)?;
            let topic_len: u16 = u16::from_be_bytes(topic_len);
            let topic_idx: u16 = reader.stream_position();
            reader
                .seek(SeekFrom::Current(topic_len.try_into().unwrap_or(i16::MAX)))
                .map_err(Error::map_w5500)?;

            let mut property_len: [u8; 1] = [0];
            reader
                .read_exact(&mut property_len)
                .map_err(Error::map_w5500)?;
            let property_len: u8 = property_len[0];
            if property_len != 0 {
                warn!("ignoring PUBLISH properties");
                reader
                    .seek(SeekFrom::Current(property_len.into()))
                    .map_err(Error::map_w5500)?;
            }

            let payload_len: u16 = header
                .remaining_len
                .saturating_sub(topic_len)
                .saturating_sub(TOPIC_LEN_LEN)
                .saturating_sub(PROPERTY_LEN_LEN)
                .saturating_sub(u16::from(property_len));
            let payload_idx: u16 = reader.stream_position();

            Ok(Some(Event::Publish(PublishReader {
                reader,
                topic_len,
                topic_idx,
                payload_len,
                payload_idx,
                _reader_error: Default::default(),
            })))
        }
        x => {
            warn!("Unhandled control packet: {:?}", x);
            reader
                .seek(SeekFrom::Current(
                    header.remaining_len.try_into().unwrap_or(i16::MAX),
                ))
                .map_err(Error::map_w5500)?;
            reader.done()?;
            Ok(None)
        }
    }
}
