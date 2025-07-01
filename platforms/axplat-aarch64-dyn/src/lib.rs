#![cfg(target_arch = "aarch64")]
#![no_std]

#[macro_use]
extern crate axplat;
extern crate alloc;

use pie_boot::BootInfo;

mod console;
mod driver;
mod init;
#[cfg(feature = "irq")]
mod irq;
mod mem;
mod power;
mod time;

mod config {
    axconfig_macros::include_configs!(path_env = "AX_CONFIG_PATH", fallback = "axconfig.toml");
}

#[pie_boot::entry]
fn main(args: &BootInfo) -> ! {
    axplat::call_main(0, args.fdt.map(|p| p.as_ptr() as usize).unwrap_or_default());
}
