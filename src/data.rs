#[cfg(feature = "serde_derive")]
include!("data.rs.in");

#[cfg(not(feature = "serde_derive"))]
include!(concat!(env!("OUT_DIR"), "/data.rs"));
