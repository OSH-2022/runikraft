//平台

#[cfg(target_arch = "riscv64")]
#[macro_use]
mod riscv64;

#[cfg(target_arch = "riscv64")]
pub use riscv64::*;

