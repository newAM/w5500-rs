use embedded_hal_mock as hal;
use w5500_ll::blocking::fdm::W5500;
use w5500_ll::Registers;

#[test]
fn fdm_nothing() {
    let spi = hal::spi::Mock::new(&[]);
    let mut w5500 = W5500::new(spi);
    w5500.write(0, 0, &[]).unwrap();
    w5500.read(0, 0, &mut []).unwrap();
}

#[test]
fn fdm_write_1() {
    const DATA: u8 = 0xAB;
    let spi = hal::spi::Mock::new(&[
        hal::spi::Transaction::write(vec![0x00, 0x00, 0b101]),
        hal::spi::Transaction::write(vec![DATA]),
    ]);
    let mut w5500 = W5500::new(spi);
    w5500.write(0, 0, &[DATA]).unwrap();
}

#[test]
fn fdm_write_2() {
    const DATA: [u8; 2] = [0xAB, 0xCD];
    let spi = hal::spi::Mock::new(&[
        hal::spi::Transaction::write(vec![0x00, 0x00, 0b110]),
        hal::spi::Transaction::write(Vec::from(DATA)),
    ]);
    let mut w5500 = W5500::new(spi);
    w5500.write(0, 0, &DATA).unwrap();
}

#[test]
fn fdm_write_4() {
    const DATA: [u8; 4] = [0x01, 0x23, 0x45, 0x56];
    let spi = hal::spi::Mock::new(&[
        hal::spi::Transaction::write(vec![0x00, 0x00, 0b111]),
        hal::spi::Transaction::write(Vec::from(DATA)),
    ]);
    let mut w5500 = W5500::new(spi);
    w5500.write(0, 0, &DATA).unwrap();
}

#[test]
fn fdm_write_chunk() {
    const DATA: [u8; 7] = [0x01, 0x23, 0x45, 0x56, 0x78, 0x9A, 0xBC];
    const ADDRESS: u16 = 0xFFFE;
    let spi = hal::spi::Mock::new(&[
        hal::spi::Transaction::write(vec![0xFF, 0xFE, 0b111]),
        hal::spi::Transaction::write(vec![0x01, 0x23, 0x45, 0x56]),
        hal::spi::Transaction::write(vec![0x00, 0x02, 0b110]),
        hal::spi::Transaction::write(vec![0x78, 0x9A]),
        hal::spi::Transaction::write(vec![0x00, 0x04, 0b101]),
        hal::spi::Transaction::write(vec![0xBC]),
    ]);
    let mut w5500 = W5500::new(spi);
    w5500.write(ADDRESS, 0, &DATA).unwrap();
}

#[test]
fn fdm_read_1() {
    const DATA_IN: u8 = 0xAB;
    const DATA_OUT: u8 = 0xEF;
    let spi = hal::spi::Mock::new(&[
        hal::spi::Transaction::write(vec![0x00, 0x00, 0b01]),
        hal::spi::Transaction::transfer(vec![DATA_IN], vec![DATA_OUT]),
    ]);
    let mut w5500 = W5500::new(spi);
    let mut buf: [u8; 1] = [DATA_IN];
    w5500.read(0, 0, &mut buf).unwrap();
    assert_eq!(buf[0], DATA_OUT);
}

#[test]
fn fdm_read_2() {
    const DATA_IN: [u8; 2] = [0xAB, 0xCD];
    const DATA_OUT: [u8; 2] = [0x12, 0x34];
    let spi = hal::spi::Mock::new(&[
        hal::spi::Transaction::write(vec![0x00, 0x00, 0b10]),
        hal::spi::Transaction::transfer(Vec::from(DATA_IN), Vec::from(DATA_OUT)),
    ]);
    let mut w5500 = W5500::new(spi);
    let mut buf: [u8; 2] = DATA_IN;
    w5500.read(0, 0, &mut buf).unwrap();
    assert_eq!(buf, DATA_OUT);
}

#[test]
fn fdm_read_4() {
    const DATA_IN: [u8; 4] = [0x12, 0x34, 0x56, 0x78];
    const DATA_OUT: [u8; 4] = [0x9A, 0xBC, 0xDE, 0xF0];
    let spi = hal::spi::Mock::new(&[
        hal::spi::Transaction::write(vec![0x00, 0x00, 0b11]),
        hal::spi::Transaction::transfer(Vec::from(DATA_IN), Vec::from(DATA_OUT)),
    ]);
    let mut w5500 = W5500::new(spi);
    let mut buf: [u8; 4] = DATA_IN;
    w5500.read(0, 0, &mut buf).unwrap();
    assert_eq!(buf, DATA_OUT);
}

#[test]
fn fdm_read_chunk() {
    const DATA: [u8; 7] = [0x01, 0x23, 0x45, 0x56, 0x78, 0x9A, 0xBC];
    const ADDRESS: u16 = 0xFFFA;
    let spi = hal::spi::Mock::new(&[
        hal::spi::Transaction::write(vec![0xFF, 0xFA, 0b11]),
        hal::spi::Transaction::transfer(vec![0; 4], vec![0x01, 0x23, 0x45, 0x56]),
        hal::spi::Transaction::write(vec![0xFF, 0xFE, 0b10]),
        hal::spi::Transaction::transfer(vec![0; 2], vec![0x78, 0x9A]),
        hal::spi::Transaction::write(vec![0x00, 0x00, 0b01]),
        hal::spi::Transaction::transfer(vec![0; 1], vec![0xBC]),
    ]);
    let mut w5500 = W5500::new(spi);
    let mut buf: [u8; 7] = [0; 7];
    w5500.read(ADDRESS, 0, &mut buf).unwrap();
    assert_eq!(buf, DATA);
}
