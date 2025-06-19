#![cfg(target_arch = "aarch64")]
#![no_std]

#[macro_use]
extern crate axplat;

use pie_boot::BootInfo;
use spin::Once;

mod console;
mod init;
mod irq;
mod mem;
mod power;
mod time;

mod config {
    axconfig_macros::include_configs!(path_env = "AX_CONFIG_PATH", fallback = "axconfig.toml");
}

// Use `.data` section to prevent being cleaned by `clean_bss`.
#[unsafe(link_section = ".data")]
static BOOT_INFO: Once<BootInfo> = Once::new();

#[pie_boot::entry]
fn main(args: &BootInfo) -> ! {
    BOOT_INFO.call_once(move || args.clone());

    axplat::call_main(
        args.cpu_id,
        args.fdt.map(|p| p.as_ptr() as usize).unwrap_or_default(),
    );
}

fn boot_info() -> &'static BootInfo {
    BOOT_INFO.wait()
}
