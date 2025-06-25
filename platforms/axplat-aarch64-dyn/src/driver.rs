use core::ops::Deref;

use pie_boot::boot_info;
use rdrive::{
    Platform, init,
    register::{DriverRegister, DriverRegisterSlice},
    register_append,
};

pub fn setup() {
    let fdt = boot_info().fdt.expect("FDT must be present");

    init(Platform::Fdt { addr: fdt }).unwrap();

    register_append(&driver_registers());
}

fn driver_registers() -> impl Deref<Target = [DriverRegister]> {
    unsafe extern "C" {
        fn __sdriver_register();
        fn __edriver_register();
    }

    unsafe {
        let len = __edriver_register as usize - __sdriver_register as usize;

        if len == 0 {
            return DriverRegisterSlice::empty();
        }

        let data = core::slice::from_raw_parts(__sdriver_register as _, len);

        DriverRegisterSlice::from_raw(data)
    }
}
