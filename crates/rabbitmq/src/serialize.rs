#[cfg(feature = "consumer")]
use std::io::Read;
#[cfg(feature = "producer")]
use std::io::Write;

/// Serialize a message into a [`Write`] stream
///
/// # Errors
/// This function fails if an I/O error occurs or a wire format error occurs.
#[cfg(feature = "producer")]
pub fn serialize<M: serde::Serialize>(
    w: impl Write,
    msg: &M,
) -> Result<(), rmp_serde::encode::Error> {
    let mut ser = rmp_serde::Serializer::new(w)
        .with_binary()
        .with_struct_map();

    msg.serialize(&mut ser)
}

/// Deserialize a message from a [`Read`] stream
///
/// # Errors
/// This function fails if an I/O error occurs or a wire format error occurs.
#[cfg(feature = "consumer")]
pub fn deserialize<M: for<'a> serde::Deserialize<'a>>(
    r: impl Read,
) -> Result<M, rmp_serde::decode::Error> {
    let mut de = rmp_serde::Deserializer::new(r).with_binary();

    M::deserialize(&mut de)
}
