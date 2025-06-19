#![cfg(all(target_arch = "aarch64", target_os = "none"))]
#![no_std]

#[macro_use]
extern crate axplat;

use pie_boot::BootArgs;

mod console;
mod init;
mod irq;
mod mem;
mod power;
mod time;

mod config {
    axconfig_macros::include_configs!(path_env = "AX_CONFIG_PATH", fallback = "axconfig.toml");
}

#[pie_boot::entry]
fn main(_args: &BootArgs) -> ! {
    // TODO: Implement actual bootstrap logic
    axplat::call_main(0, 0);
}
