pub type SharedSecret = [u8; 32];

cfg_if::cfg_if! {
    if #[cfg(feature = "p256-cm4")] {
        mod cm4;
        pub use cm4::*;
    } else {
        mod rust_crypto;
        pub use rust_crypto::*;
    }
}
