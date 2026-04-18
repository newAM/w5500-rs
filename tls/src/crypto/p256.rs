pub type SharedSecret = [u8; 32];

cfg_select! {
    feature = "p256-cm4" => {
        mod cm4;
        pub use cm4::*;
    }
    _ => {
        mod rust_crypto;
        pub use rust_crypto::*;
    }
}
