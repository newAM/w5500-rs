use crate::{
    crypto::encrypt_record_inplace, handshake::HandshakeHeader, key_schedule::KeySchedule,
    AlertDescription, ContentType,
};
use core::{
    cmp::{self, min},
    convert::Infallible,
};
use sha2::{Digest, Sha256};
use w5500_hl::{
    io::{Read, Seek, SeekFrom, Write},
    ll::{Registers, Sn},
    Error as HlError,
};

/// Helper to read from a circular buffer expressed as two memory ranges
/// `a` and `b`.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
pub(crate) struct CircleReader<'a> {
    a: &'a [u8],
    b: &'a [u8],
    ptr: u16,
}

impl<'a> CircleReader<'a> {
    pub const fn new(a: &'a [u8], b: &'a [u8]) -> Self {
        Self { a, b, ptr: 0 }
    }

    pub fn len(&self) -> u16 {
        (self.a.len() + self.b.len()) as u16
    }

    fn as_slices(&self) -> (&[u8], &[u8]) {
        if let Some(b_ptr) = usize::from(self.ptr).checked_sub(self.a.len()) {
            (&self.b[b_ptr..], &[])
        } else {
            (&self.a[self.ptr.into()..], self.b)
        }
    }

    fn as_slices_of_n(&self, n: usize) -> Option<(&[u8], &[u8])> {
        let (a, b): (&[u8], &[u8]) = self.as_slices();
        if a.len() + b.len() < n {
            None
        } else if a.len() >= n {
            Some((&a[..n], &[]))
        } else {
            let b_n: usize = n - a.len();
            Some((a, &b[..b_n]))
        }
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), AlertDescription> {
        let (a, b): (&[u8], &[u8]) = self
            .as_slices_of_n(buf.len())
            .ok_or(AlertDescription::DecodeError)?;

        buf[..a.len()].copy_from_slice(a);
        buf[a.len()..].copy_from_slice(b);

        self.ptr += buf.len() as u16;

        Ok(())
    }

    #[must_use]
    pub fn read(&mut self, buf: &mut [u8]) -> u16 {
        let (a, b): (&[u8], &[u8]) = self.as_slices();

        let copy_a: usize = min(a.len(), buf.len());
        buf[..copy_a].copy_from_slice(&a[..copy_a]);

        let copy_b: usize = min(b.len(), buf.len().saturating_sub(a.len()));
        buf[copy_a..(copy_a + copy_b)].copy_from_slice(&b[..copy_b]);

        let read_len: u16 = (copy_a + copy_b) as u16;

        self.ptr += read_len;

        read_len
    }

    pub fn stream_position(&self) -> u16 {
        self.ptr as u16
    }

    pub fn next_n<const N: usize>(&mut self) -> Result<[u8; N], AlertDescription> {
        let mut buf: [u8; N] = [0; N];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn next_u8(&mut self) -> Result<u8, AlertDescription> {
        let data: [u8; 1] = self.next_n()?;
        Ok(data[0])
    }

    pub fn next_u16(&mut self) -> Result<u16, AlertDescription> {
        Ok(u16::from_be_bytes(self.next_n()?))
    }

    pub fn skip_n(&mut self, n: u16) -> Result<(), AlertDescription> {
        match self.ptr.checked_add(n) {
            Some(next_ptr) if next_ptr <= self.len() => {
                self.ptr = next_ptr;
                Ok(())
            }
            _ => Err(AlertDescription::DecodeError),
        }
    }
}

/// Writer for a TLS application data record.
///
/// This implements the `w5500-hl` IO traits, [`Write`] and [`Seek`].
///
/// Created by [`Client::writer`].
///
/// This writes plaintext to the socket buffers. When [`Write::send`] is called
/// the data in the socket buffers is encrypted, and has the appropriate headers
/// and footers added.
///
/// # Example
///
/// ```no_run
/// # let mut rng = rand_core::OsRng;
/// # fn monotonic_secs() -> u32 { 0 }
/// # const MY_KEY: [u8; 1] = [0];
/// # let mut w5500 = w5500_regsim::W5500::default();
/// use w5500_tls::{
///     Client, Event, TlsWriter,
///     {
///         hl::{io::Write, Hostname},
///         ll::{
///             net::{Ipv4Addr, SocketAddrV4},
///             Sn,
///         },
///     },
/// };
///
/// static mut RX: [u8; 2048] = [0; 2048];
///
/// const DST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 4), 8883);
/// const HOSTNAME: Hostname = Hostname::new_unwrapped("server.local");
/// const SRC_PORT: u16 = 1234;
/// const TLS_SN: Sn = Sn::Sn4;
///
/// let mut tls_client: Client<2048> = Client::new(
///     TLS_SN,
///     SRC_PORT,
///     HOSTNAME,
///     DST,
///     b"mykeyidentity",
///     &MY_KEY,
///     unsafe { &mut RX },
/// );
///
/// // wait until the handshake has completed
/// // for demonstration purposes only, please do not actually do this
/// while tls_client.process(&mut w5500, &mut rng, monotonic_secs()) != Ok(Event::HandshakeFinished)
/// {
/// }
///
/// let mut writer: TlsWriter<_> = tls_client.writer(&mut w5500).unwrap();
/// writer.write_all(&[0xAA; 5])?;
/// writer.send()?;
/// # Ok::<(), w5500_hl::Error<_>>(())
/// ```
///
/// [`Client::writer`]: crate::Client::writer
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
pub struct TlsWriter<'w, 'ks, W5500: Registers> {
    pub(crate) w5500: &'w mut W5500,
    pub(crate) key_schedule: &'ks mut KeySchedule,
    pub(crate) sn: Sn,
    pub(crate) head_ptr: u16,
    pub(crate) tail_ptr: u16,
    pub(crate) ptr: u16,
}

impl<'w, 'ks, W5500: Registers> Seek<W5500::Error> for TlsWriter<'w, 'ks, W5500> {
    fn seek(&mut self, pos: SeekFrom) -> Result<(), HlError<W5500::Error>> {
        self.ptr = pos.new_ptr(self.ptr, self.head_ptr, self.tail_ptr)?;
        Ok(())
    }

    #[inline]
    fn rewind(&mut self) {
        self.ptr = self.head_ptr
    }

    #[inline]
    fn stream_len(&self) -> u16 {
        self.tail_ptr.wrapping_sub(self.head_ptr)
    }

    #[inline]
    fn stream_position(&self) -> u16 {
        self.ptr.wrapping_sub(self.head_ptr)
    }

    #[inline]
    fn remain(&self) -> u16 {
        self.tail_ptr.wrapping_sub(self.ptr)
    }
}

impl<'w, 'ks, W5500: Registers> Write<W5500::Error> for TlsWriter<'w, 'ks, W5500> {
    fn write(&mut self, buf: &[u8]) -> Result<u16, W5500::Error> {
        let write_size: u16 = min(self.remain(), buf.len().try_into().unwrap_or(u16::MAX));
        if write_size != 0 {
            self.w5500
                .set_sn_tx_buf(self.sn, self.ptr, &buf[..usize::from(write_size)])?;
            self.ptr = self.ptr.wrapping_add(write_size);

            Ok(write_size)
        } else {
            Ok(0)
        }
    }

    fn write_all(&mut self, buf: &[u8]) -> Result<(), HlError<W5500::Error>> {
        let buf_len: u16 = buf.len().try_into().unwrap_or(u16::MAX);
        let write_size: u16 = min(self.remain(), buf_len);
        if write_size != buf_len {
            Err(HlError::OutOfMemory)
        } else {
            self.w5500.set_sn_tx_buf(self.sn, self.ptr, buf)?;
            self.ptr = self.ptr.wrapping_add(write_size);
            Ok(())
        }
    }

    fn send(self) -> Result<(), W5500::Error> {
        let (key, nonce): ([u8; 16], [u8; 12]) = self.key_schedule.client_key_and_nonce().unwrap();
        encrypt_record_inplace(
            self.w5500,
            self.sn,
            &key,
            &nonce,
            self.head_ptr,
            self.ptr,
            ContentType::ApplicationData,
        )?;
        self.key_schedule.increment_write_record_sequence_number();
        Ok(())
    }
}

/// Reader for a TLS application data record.
///
/// This implements the `w5500-hl` IO traits, [`Read`] and [`Seek`].
///
/// Created by [`Client::reader`].
///
/// [`Client::reader`]: crate::Client::reader
///
/// # Example
///
/// ```no_run
/// # let mut rng = rand_core::OsRng;
/// # fn monotonic_secs() -> u32 { 0 }
/// # const MY_KEY: [u8; 1] = [0];
/// # let mut w5500 = w5500_regsim::W5500::default();
/// use w5500_tls::{
///     Client, Event, TlsReader,
///     {
///         hl::{io::Read, Hostname},
///         ll::{
///             net::{Ipv4Addr, SocketAddrV4},
///             Sn,
///         },
///     },
/// };
///
/// static mut RX: [u8; 2048] = [0; 2048];
///
/// const DST: SocketAddrV4 = SocketAddrV4::new(Ipv4Addr::new(192, 168, 0, 4), 8883);
/// const HOSTNAME: Hostname = Hostname::new_unwrapped("server.local");
/// const SRC_PORT: u16 = 1234;
/// const TLS_SN: Sn = Sn::Sn4;
///
/// let mut tls_client: Client<2048> = Client::new(
///     TLS_SN,
///     SRC_PORT,
///     HOSTNAME,
///     DST,
///     b"mykeyidentity",
///     &MY_KEY,
///     unsafe { &mut RX },
/// );
///
/// // wait until there is application data
/// // for demonstration purposes only, please do not actually do this
/// while tls_client.process(&mut w5500, &mut rng, monotonic_secs()) != Ok(Event::ApplicationData) {
/// }
///
/// let mut reader: TlsReader = tls_client.reader()?;
/// let mut buf: [u8; 5] = [0; 5];
/// reader.read_exact(&mut buf)?;
/// reader.done()?;
/// # Ok::<(), w5500_hl::Error<_>>(())
/// ```
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "ufmt", derive(ufmt::derive::uDebug))]
pub struct TlsReader<'buf, 'ptr> {
    inner: CircleReader<'buf>,
    head: &'ptr mut usize,
    wrap: usize,
}

// TODO: use wrapping_add_signed when stabilized
// https://github.com/rust-lang/rust/issues/87840
// https://github.com/rust-lang/rust/blob/21b0325c68421b00c6c91055ac330bd5ffe1ea6b/library/core/src/num/uint_macros.rs#L1205
fn wrapping_add_signed(ptr: u16, offset: i16) -> u16 {
    ptr.wrapping_add(offset as u16)
}

impl<'buf, 'ptr> Seek<Infallible> for TlsReader<'buf, 'ptr> {
    fn seek(&mut self, pos: SeekFrom) -> Result<(), HlError<Infallible>> {
        match pos {
            SeekFrom::Start(n) => {
                if n > self.stream_len() {
                    Err(HlError::UnexpectedEof)
                } else {
                    self.inner.ptr = n;
                    Ok(())
                }
            }
            SeekFrom::End(n) => {
                if n > 0 {
                    Err(HlError::UnexpectedEof)
                } else {
                    let n_abs: u16 = n.abs_diff(0);
                    let ptr: u16 = self
                        .stream_len()
                        .checked_sub(n_abs)
                        .ok_or(HlError::UnexpectedEof)?;
                    self.inner.ptr = ptr;
                    Ok(())
                }
            }
            SeekFrom::Current(offset) => {
                let max_val: i16 = self
                    .stream_len()
                    .saturating_sub(self.inner.ptr)
                    .try_into()
                    .unwrap_or(i16::MAX);
                let min_val: i16 = 0_i16
                    .checked_sub(i16::try_from(self.inner.ptr).unwrap_or(i16::MAX))
                    .unwrap_or(i16::MIN);

                if offset < min_val || offset > max_val {
                    Err(HlError::UnexpectedEof)
                } else {
                    self.inner.ptr = wrapping_add_signed(self.inner.ptr, offset);
                    Ok(())
                }
            }
        }
    }

    #[inline]
    fn rewind(&mut self) {
        self.inner.ptr = 0;
    }

    #[inline]
    fn stream_len(&self) -> u16 {
        self.inner.len()
    }

    #[inline]
    fn stream_position(&self) -> u16 {
        self.inner.ptr
    }

    #[inline]
    fn remain(&self) -> u16 {
        self.stream_len() - self.inner.ptr
    }
}

impl<'buf, 'ptr> Read<Infallible> for TlsReader<'buf, 'ptr> {
    fn read(&mut self, buf: &mut [u8]) -> Result<u16, Infallible> {
        Ok(self.inner.read(buf))
    }

    fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), HlError<Infallible>> {
        self.inner
            .read_exact(buf)
            .map_err(|_| HlError::UnexpectedEof)
    }

    fn done(self) -> Result<(), Infallible> {
        *self.head = (*self.head + usize::from(self.inner.ptr)) % self.wrap;
        Ok(())
    }
}

/// Circular buffer
///
/// All data from the W5500 socket RX buffer gets placed into this buffer.
///
/// This is a little weird because it needs to handle two kinds of content
/// types: Application data, and everything else.
/// Application data gets handled by the user, and everything else is handled
/// internally.
///
/// To maintain a continuous buffer of application data the buffer pointers are
/// rewound to the head of the application data after non-application
/// data records have been reassembled and processed.
///
/// TLS has several fragmentation requirements that make this possible
/// [RFC 8446 Section 5.1]:
///
/// * Handshake messages MUST NOT be interleaved with other record
///   types.  That is, if a handshake message is split over two or more
///   records, there MUST NOT be any other records between them.
/// * Alert messages MUST NOT be fragmented across records.
/// * The RFC is rather unclear when it comes to `change_cipher_spec`
///   unfortunately, this is a bit of an assumption:
///   It is implied only application data records can be zero-length.
///   This in turn implies `change_cipher_spec` records cannot be fragmented,
///   because they only contain a single byte.
///
/// There are several error conditions that need to be handled to avoid
/// corrupting the buffer state:
///
/// * Receive fragmented alert record.
/// * Receive fragmented `change_cipher_spec` record.
/// * Buffer contains handshake fragment, receive application data.
/// * Buffer contains application data, receive handshake fragment,
///   receive application data.
///
/// The pointers are arranged like this:
///
/// ```text
/// +-------------+  <-- buf.len()
/// | Free Space  |
/// +-------------+  <-- tail
/// | Handshake   |
/// +-------------+
/// | Handshake   |
/// +-------------+  <-- hs_head and ad_tail
/// | Application |
/// | Data        |
/// +-------------+  <-- head
/// | Free Space  |
/// +-------------+  <-- 0
/// ```
///
/// If the first handshake in the above diagram is popped this becomes:
///
/// ```text
/// +-------------+  <-- buf.len()
/// | Free Space  |
/// +-------------+  <-- tail
/// | Handshake   |
/// +-------------+  <-- hs_head
/// | Free Space  |
/// +-------------+  <-- ad_tail
/// | Application |
/// | Data        |
/// +-------------+  <-- head
/// | Free Space  |
/// +-------------+  <-- 0
/// ```
///
/// When all handshakes have been popped (`hs_head == tail`) the pointers are
/// rewound to maintain application data continuity.
///
/// ```text
/// +-------------+  <-- buf.len()
/// |             |
/// |             |
/// | Free Space  |
/// |             |
/// |             |
/// +-------------+  <-- ad_tail and hs_head and tail
/// | Application |
/// | Data        |
/// +-------------+  <-- head
/// | Free Space  |
/// +-------------+  <-- 0
/// ```
///
/// [RFC 8446 Section 5.1]: https://datatracker.ietf.org/doc/html/rfc8446#section-5.1
pub struct Buffer<'b, const N: usize> {
    buf: &'b mut [u8; N],
    head: usize,
    ad_tail: usize,
    hs_head: usize,
    tail: usize,
}

impl<'b, const N: usize> From<&'b mut [u8; N]> for Buffer<'b, N> {
    fn from(buf: &'b mut [u8; N]) -> Self {
        Self {
            buf,
            head: 0,
            ad_tail: 0,
            hs_head: 0,
            tail: 0,
        }
    }
}

// There has to be a better way to keep the borrow checker happy.
macro_rules! as_slices {
    ($buf:expr, $tail:expr, $head:expr, $n:expr $(,)*) => {{
        #[allow(unsafe_code)]
        // unsafe to avoid bounds check, bounds is tracked by head/tail pointers externally
        unsafe {
            if $tail <= $head {
                (
                    core::slice::from_raw_parts($buf.as_ptr().add($head) as *const u8, $n - $head),
                    core::slice::from_raw_parts($buf.as_ptr() as *const u8, $tail),
                )
            } else {
                (
                    core::slice::from_raw_parts(
                        $buf.as_ptr().add($head) as *const u8,
                        $tail - $head,
                    ),
                    &[],
                )
            }
        }
    }};
}

impl<'b, const N: usize> Buffer<'b, N> {
    const fn capacity(&self) -> usize {
        N - 1
    }

    fn len(&self) -> usize {
        if self.tail < self.head {
            self.tail + self.capacity() - self.head
        } else {
            self.tail - self.head
        }
    }

    fn hs_len(&self) -> usize {
        if self.tail < self.hs_head {
            self.tail + self.capacity() - self.hs_head
        } else {
            self.tail - self.hs_head
        }
    }

    pub fn reset(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.ad_tail = 0;
        self.hs_head = 0;
    }

    pub fn contains_handshake_fragment(&self) -> bool {
        self.tail != self.ad_tail
    }

    pub fn increment_application_data_tail(&mut self, n: usize) {
        debug_assert!(n <= self.capacity(), "{} <= {}", n, self.capacity());
        self.ad_tail = (self.ad_tail + n) % N;
    }

    fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    fn remain(&self) -> usize {
        self.capacity() - self.len()
    }

    fn pop_handshake_slices_of_n(&mut self, n: usize) -> Option<(&[u8], &[u8])> {
        if self.len() < n {
            None
        } else {
            let (a, b): (&[u8], &[u8]) = as_slices!(self.buf, self.tail, self.hs_head, N);
            self.hs_head = (self.hs_head + n) % N;
            if self.hs_head == self.tail {
                self.hs_head = self.ad_tail;
                self.tail = self.ad_tail;
            }
            if a.len() >= n {
                Some((&a[..n], &[]))
            } else {
                let b_n: usize = n - a.len();
                Some((a, &b[..b_n]))
            }
        }
    }

    /// Push slice to tail
    pub fn extend_from_slice(&mut self, src: &[u8]) -> Result<(), AlertDescription> {
        if src.len() > self.remain() {
            debug!("src.len > remain; {} > {}", src.len(), self.remain());
            Err(AlertDescription::InternalError)
        } else {
            let (a, b): (&mut [u8], &mut [u8]) = match self.tail.cmp(&self.head) {
                cmp::Ordering::Equal => {
                    let (b, a): (&mut [u8], &mut [u8]) = self.buf.split_at_mut(self.tail);
                    (a, b)
                }
                cmp::Ordering::Greater => {
                    let (remain, a): (&mut [u8], &mut [u8]) = self.buf.split_at_mut(self.tail);
                    let (b, _): (&mut [u8], &mut [u8]) = remain.split_at_mut(self.head);
                    (a, b)
                }
                cmp::Ordering::Less => (&mut self.buf[self.tail..self.head], &mut []),
            };

            let a_len: usize = min(src.len(), a.len());
            let (src_a, src_b): (&[u8], &[u8]) = src.split_at(a_len);

            a[..src_a.len()].copy_from_slice(src_a);
            b[..src_b.len()].copy_from_slice(src_b);

            self.tail = (self.tail + src.len()) % N;

            if !src.is_empty() {
                debug_assert_ne!(self.head, self.tail);
            }

            Ok(())
        }
    }

    /// Pop content type from tail.
    pub fn pop_tail(&mut self) -> Option<u8> {
        if self.is_empty() {
            None
        } else {
            self.tail = self.tail.checked_sub(1).unwrap_or(N - 1);
            Some(self.buf[self.tail])
        }
    }

    /// Read `LEN` bytes from the head, without removing it from the queue.
    fn read_head<const LEN: usize>(&self) -> Option<[u8; LEN]> {
        if self.len() < LEN {
            None
        } else {
            let mut ret: [u8; LEN] = [0; LEN];
            let mut tmp_head = self.head;
            for byte in ret.iter_mut() {
                *byte = self.buf[tmp_head];
                tmp_head += 1;
                if tmp_head == N {
                    tmp_head = 0;
                }
            }
            Some(ret)
        }
    }

    pub(crate) fn pop_handshake_record(
        &mut self,
        hash: &mut Sha256,
    ) -> Result<Option<(HandshakeHeader, CircleReader)>, AlertDescription> {
        let hs_hdr: [u8; HandshakeHeader::LEN] = match self.read_head() {
            Some(hs_hdr) => hs_hdr,
            // fragment is not long enough to contain handshake type + length
            None => return Ok(None),
        };

        let hs_hdr: HandshakeHeader = hs_hdr.into();

        debug!("Handshake.msg_type={:?}", hs_hdr.msg_type());
        debug!("Handshake.length={:?}", hs_hdr.length());

        if hs_hdr.length_with_header() > self.capacity() as u32 {
            error!(
                "RX buffer is not long enough for handshake {}",
                hs_hdr.length_with_header()
            );
            return Err(AlertDescription::InternalError);
        }

        // fragment is not long enough to contain entire handshake
        if hs_hdr.length_with_header() > u32::try_from(self.hs_len()).unwrap_or(u32::MAX) {
            debug!("handshake is fragmented");
            return Ok(None);
        }

        // "pop" handshake header from the buffer head
        self.hs_head = (self.hs_head + HandshakeHeader::LEN) % N;

        let (a, b): (&[u8], &[u8]) = self
            .pop_handshake_slices_of_n(hs_hdr.length() as usize)
            .unwrap();
        hash.update(hs_hdr.as_bytes());
        hash.update(a);
        hash.update(b);
        Ok(Some((hs_hdr, CircleReader::new(a, b))))
    }

    // used for sending ClientHello
    pub fn as_mut_buf(&mut self) -> &mut [u8; N] {
        debug_assert_eq!(self.head, 0);
        debug_assert_eq!(self.tail, 0);
        self.buf
    }

    // used for sending ClientHello
    pub fn as_buf(&mut self) -> &mut [u8; N] {
        debug_assert_eq!(self.head, 0);
        debug_assert_eq!(self.tail, 0);
        self.buf
    }

    pub fn app_data_reader<'a>(&'a mut self) -> Result<TlsReader<'b, 'a>, HlError<Infallible>> {
        if self.ad_tail == self.head {
            Err(HlError::WouldBlock)
        } else {
            let (a, b): (&'b [u8], &'b [u8]) = as_slices!(self.buf, self.ad_tail, self.head, N);

            Ok(TlsReader {
                inner: CircleReader::new(a, b),
                head: &mut self.head,
                wrap: N,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, Read};

    #[test]
    fn basic() {
        let mut buf: [u8; 6] = [0; 6];
        let mut buffer = Buffer::from(&mut buf);

        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.remain(), 5);
        assert!(buffer.is_empty());

        let a: [u8; 2] = [0x01, 0x23];
        let b: [u8; 2] = [0x45, 0x67];
        let c: [u8; 2] = [0x78, 0x98];
        buffer.extend_from_slice(&a).unwrap();
        assert_eq!(buffer.len(), 2);
        assert_eq!(buffer.remain(), 3);
        buffer.extend_from_slice(&b).unwrap();
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.remain(), 1);

        buffer.extend_from_slice(&c).unwrap_err();
        assert_eq!(buffer.len(), 4);
        assert_eq!(buffer.remain(), 1);

        assert_eq!(buffer.pop_tail(), Some(0x67));
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.remain(), 2);

        buffer.extend_from_slice(&c).unwrap();
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.remain(), 0);
    }

    #[test]
    fn extend_from_slice_wrap() {
        let mut buf: [u8; 6] = [0; 6];
        let mut buffer = Buffer::from(&mut buf);

        const APP_DATA: [u8; 5] = [0x01, 0x23, 0x45, 0x67, 0x89];

        buffer.extend_from_slice(&APP_DATA).unwrap();
        buffer.increment_application_data_tail(APP_DATA.len());

        // only reader increments the head pointer
        let mut reader = buffer.app_data_reader().unwrap();
        let mut buf: [u8; APP_DATA.len()] = [0; APP_DATA.len()];
        reader.read_exact(&mut buf).unwrap();
        reader.done().unwrap();

        buffer.extend_from_slice(&[0x67, 0x89, 0xAB]).unwrap();
    }
}
