#![cfg(target_arch = "aarch64")]
#![no_std]

#[macro_use]
extern crate axplat;

use pie_boot::BootInfo;

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
fn main(args: &BootInfo) -> ! {
    axplat::call_main(
        args.cpu_id,
        args.fdt.map(|p| p.as_ptr() as usize).unwrap_or_default(),
    );
}
