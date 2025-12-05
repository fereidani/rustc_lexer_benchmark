#![deny(unstable_features)]

#[cfg(feature = "orig")]
include!("../src_orig/lib.rs");

#[cfg(not(feature = "orig"))]
include!("../src_new/lib.rs");
