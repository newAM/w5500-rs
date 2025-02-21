use w5500_ll::Registers;
use w5500_ll::eh1::fdm::W5500;

#[test]
fn fdm_nothing() {
    let spi = ehm::eh1::spi::Mock::new(&[]);
    let mut w5500 = W5500::new(spi);
    w5500.write(0, 0, &[]).unwrap();
    w5500.read(0, 0, &mut []).unwrap();
    w5500.free().done();
}

#[test]
fn fdm_write_1() {
    const DATA: u8 = 0xAB;
    let spi = ehm::eh1::spi::Mock::new(&[
        ehm::eh1::spi::Transaction::write_vec(vec![0x00, 0x00, 0b101]),
        ehm::eh1::spi::Transaction::write_vec(vec![DATA]),
    ]);
    let mut w5500 = W5500::new(spi);
    w5500.write(0, 0, &[DATA]).unwrap();
    w5500.free().done();
}

#[test]
fn fdm_write_2() {
    const DATA: [u8; 2] = [0xAB, 0xCD];
    let spi = ehm::eh1::spi::Mock::new(&[
        ehm::eh1::spi::Transaction::write_vec(vec![0x00, 0x00, 0b110]),
        ehm::eh1::spi::Transaction::write_vec(Vec::from(DATA)),
    ]);
    let mut w5500 = W5500::new(spi);
    w5500.write(0, 0, &DATA).unwrap();
    w5500.free().done();
}

#[test]
fn fdm_write_4() {
    const DATA: [u8; 4] = [0x01, 0x23, 0x45, 0x56];
    let spi = ehm::eh1::spi::Mock::new(&[
        ehm::eh1::spi::Transaction::write_vec(vec![0x00, 0x00, 0b111]),
        ehm::eh1::spi::Transaction::write_vec(Vec::from(DATA)),
    ]);
    let mut w5500 = W5500::new(spi);
    w5500.write(0, 0, &DATA).unwrap();
    w5500.free().done();
}

#[test]
fn fdm_write_chunk() {
    const DATA: [u8; 7] = [0x01, 0x23, 0x45, 0x56, 0x78, 0x9A, 0xBC];
    const ADDRESS: u16 = 0xFFFE;
    let spi = ehm::eh1::spi::Mock::new(&[
        ehm::eh1::spi::Transaction::write_vec(vec![0xFF, 0xFE, 0b111]),
        ehm::eh1::spi::Transaction::write_vec(vec![0x01, 0x23, 0x45, 0x56]),
        ehm::eh1::spi::Transaction::write_vec(vec![0x00, 0x02, 0b110]),
        ehm::eh1::spi::Transaction::write_vec(vec![0x78, 0x9A]),
        ehm::eh1::spi::Transaction::write_vec(vec![0x00, 0x04, 0b101]),
        ehm::eh1::spi::Transaction::write_vec(vec![0xBC]),
    ]);
    let mut w5500 = W5500::new(spi);
    w5500.write(ADDRESS, 0, &DATA).unwrap();
    w5500.free().done();
}

#[test]
fn fdm_read_1() {
    const DATA_OUT: u8 = 0xEF;
    let spi = ehm::eh1::spi::Mock::new(&[
        ehm::eh1::spi::Transaction::write_vec(vec![0x00, 0x00, 0b01]),
        ehm::eh1::spi::Transaction::read(DATA_OUT),
    ]);
    let mut w5500 = W5500::new(spi);
    let mut buf: [u8; 1] = [0];
    w5500.read(0, 0, &mut buf).unwrap();
    assert_eq!(buf[0], DATA_OUT);
    w5500.free().done();
}

#[test]
fn fdm_read_2() {
    const DATA_OUT: [u8; 2] = [0x12, 0x34];
    let spi = ehm::eh1::spi::Mock::new(&[
        ehm::eh1::spi::Transaction::write_vec(vec![0x00, 0x00, 0b10]),
        ehm::eh1::spi::Transaction::read_vec(Vec::from(DATA_OUT)),
    ]);
    let mut w5500 = W5500::new(spi);
    let mut buf: [u8; 2] = [0; 2];
    w5500.read(0, 0, &mut buf).unwrap();
    assert_eq!(buf, DATA_OUT);
    w5500.free().done();
}

#[test]
fn fdm_read_4() {
    const DATA_OUT: [u8; 4] = [0x9A, 0xBC, 0xDE, 0xF0];
    let spi = ehm::eh1::spi::Mock::new(&[
        ehm::eh1::spi::Transaction::write_vec(vec![0x00, 0x00, 0b11]),
        ehm::eh1::spi::Transaction::read_vec(Vec::from(DATA_OUT)),
    ]);
    let mut w5500 = W5500::new(spi);
    let mut buf: [u8; 4] = [0; 4];
    w5500.read(0, 0, &mut buf).unwrap();
    assert_eq!(buf, DATA_OUT);
    w5500.free().done();
}

#[test]
fn fdm_read_chunk() {
    const DATA: [u8; 7] = [0x01, 0x23, 0x45, 0x56, 0x78, 0x9A, 0xBC];
    const ADDRESS: u16 = 0xFFFA;
    let spi = ehm::eh1::spi::Mock::new(&[
        ehm::eh1::spi::Transaction::write_vec(vec![0xFF, 0xFA, 0b11]),
        ehm::eh1::spi::Transaction::read_vec(vec![0x01, 0x23, 0x45, 0x56]),
        ehm::eh1::spi::Transaction::write_vec(vec![0xFF, 0xFE, 0b10]),
        ehm::eh1::spi::Transaction::read_vec(vec![0x78, 0x9A]),
        ehm::eh1::spi::Transaction::write_vec(vec![0x00, 0x00, 0b01]),
        ehm::eh1::spi::Transaction::read_vec(vec![0xBC]),
    ]);
    let mut w5500 = W5500::new(spi);
    let mut buf: [u8; 7] = [0; 7];
    w5500.read(ADDRESS, 0, &mut buf).unwrap();
    assert_eq!(buf, DATA);
    w5500.free().done();
}
