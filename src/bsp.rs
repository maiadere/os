// board support package reexports dependent on feature flags
#[cfg(feature = "bsp_rpi4")]
mod rpi;

#[cfg(feature = "bsp_rpi4")]
pub use rpi::*;
