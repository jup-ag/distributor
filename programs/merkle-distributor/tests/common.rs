pub const PROGRAM_BYTES: &[u8] = include_bytes!(concat!(
env!("CARGO_MANIFEST_DIR"),
"/../../target/deploy/merkle_distributor.so"
));