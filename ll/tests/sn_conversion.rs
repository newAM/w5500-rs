use w5500_ll::Sn;

#[test]
fn sn_conversion() {
    assert_eq!(Sn::try_from(1_i64).unwrap(), Sn::Sn1);
    assert_eq!(usize::from(Sn::Sn7), 7_usize);
}
