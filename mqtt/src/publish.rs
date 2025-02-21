use crate::{CtrlPkt, Error, write_variable_byte_integer};
use core::{cmp::min, marker::PhantomData, mem::size_of};
use w5500_hl::{
    Error as HlError,
    io::{Read, Seek, SeekFrom, Write},
};

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

/// Reader for a published message on a subscribed topic.
///
/// This reads publish data directly from the socket buffer, avoiding the need
/// for an intermediate copy.
///
/// Created by [`Client::process`] when there is a pending message.
///
/// [`Client::process`]: crate::Client::process
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PublishReader<E, Reader: Read<E> + Seek> {
    pub(crate) reader: Reader,
    pub(crate) topic_len: u16,
    pub(crate) topic_idx: u16,
    pub(crate) payload_len: u16,
    pub(crate) payload_idx: u16,
    pub(crate) _reader_error: PhantomData<E>,
}

impl<E, Reader: Read<E> + Seek> PublishReader<E, Reader> {
    /// Length of the topic in bytes.
    #[inline]
    pub fn topic_len(&self) -> u16 {
        self.topic_len
    }

    /// Length of the payload in bytes.
    #[inline]
    pub fn payload_len(&self) -> u16 {
        self.payload_len
    }

    /// Read the topic into `buf`, and return the number of bytes read.
    pub fn read_topic(&mut self, buf: &mut [u8]) -> Result<u16, Error<E>> {
        self.reader
            .seek(SeekFrom::Start(self.topic_idx))
            .map_err(Error::map_w5500)?;
        let read_len: u16 = min(buf.len().try_into().unwrap_or(u16::MAX), self.topic_len);
        self.reader
            .read_exact(&mut buf[..read_len.into()])
            .map_err(Error::map_w5500)?;
        Ok(read_len)
    }

    /// Read the payload into `buf`, and return the number of bytes read.
    pub fn read_payload(&mut self, buf: &mut [u8]) -> Result<u16, Error<E>> {
        self.reader
            .seek(SeekFrom::Start(self.payload_idx))
            .map_err(Error::map_w5500)?;
        let read_len: u16 = min(buf.len().try_into().unwrap_or(u16::MAX), self.payload_len);
        self.reader
            .read_exact(&mut buf[..read_len.into()])
            .map_err(Error::map_w5500)?;
        Ok(read_len)
    }

    /// Mark this message as read.
    ///
    /// If this is not called the message will be returned to the queue,
    /// available upon the next call to [`Client::process`].
    ///
    /// [`Client::process`]: crate::Client::process
    #[inline]
    pub fn done(mut self) -> Result<(), Error<E>> {
        self.reader
            .seek(SeekFrom::Start(self.payload_idx + self.payload_len))
            .map_err(Error::map_w5500)?;
        self.reader.done()?;
        Ok(())
    }
}
