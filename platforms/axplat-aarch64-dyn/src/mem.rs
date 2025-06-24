use core::ops::Range;

use axplat::mem::{MemIf, RawRange};
use heapless::Vec;
use memory_addr::{PhysAddr, VirtAddr};
use pie_boot::{KIMAGE_VADDR, KLINER_OFFSET, MemoryRegionKind, boot_info};
use spin::{Mutex, Once};

struct MemIfImpl;

static RAM_LIST: Once<Vec<RawRange, 32>> = Once::new();
static RESERVED_LIST: Once<Vec<RawRange, 32>> = Once::new();
static MMIO: Mutex<Vec<RawRange, 32>> = Mutex::new(Vec::new());

pub fn setup() {
    RAM_LIST.call_once(|| {
        let mut ram_list = Vec::new();
        for region in boot_info()
            .memory_regions
            .iter()
            .filter(|one| matches!(one.kind, MemoryRegionKind::Ram))
            .map(|one| (one.start, one.end - one.start))
        {
            let _ = ram_list.push(region);
        }
        ram_list
    });

    RESERVED_LIST.call_once(|| {
        let mut ram_list = Vec::new();
        for region in boot_info()
            .memory_regions
            .iter()
            .filter(|one| {
                matches!(
                    one.kind,
                    MemoryRegionKind::Reserved | MemoryRegionKind::Bootloader
                )
            })
            .map(|one| (one.start, one.end - one.start))
        {
            let _ = ram_list.push(region);
        }
        ram_list
    });
}

#[impl_plat_interface]
impl MemIf for MemIfImpl {
    /// Returns all physical memory (RAM) ranges on the platform.
    ///
    /// All memory ranges except reserved ranges (including the kernel loaded
    /// range) are free for allocation.
    fn phys_ram_ranges() -> &'static [RawRange] {
        RAM_LIST.wait()
    }

    /// Returns all reserved physical memory ranges on the platform.
    ///
    /// Reserved memory can be contained in [`phys_ram_ranges`], they are not
    /// allocatable but should be mapped to kernel's address space.
    ///
    /// Note that the ranges returned should not include the range where the
    /// kernel is loaded.
    fn reserved_phys_ram_ranges() -> &'static [RawRange] {
        RESERVED_LIST.wait()
    }

    /// Returns all device memory (MMIO) ranges on the platform.
    fn mmio_ranges() -> &'static [RawRange] {
        &[]
    }

    fn phys_to_virt(p: PhysAddr) -> VirtAddr {
        if kimage_range_phys().contains(&p) {
            VirtAddr::from_usize(p.as_usize() + KIMAGE_VADDR)
        } else {
            // MMIO or other reserved regions
            VirtAddr::from_usize(p.as_usize() + KLINER_OFFSET)
        }
    }
    fn virt_to_phys(p: VirtAddr) -> PhysAddr {
        if (KIMAGE_VADDR..KLINER_OFFSET).contains(&p.as_usize()) {
            PhysAddr::from_usize(p.as_usize() - KIMAGE_VADDR)
        } else {
            PhysAddr::from_usize(p.as_usize() - KLINER_OFFSET)
        }
    }
}

fn kimage_range_phys() -> Range<PhysAddr> {
    unsafe extern "C" {
        fn _skernel();
        fn _ekernel();
    }

    let start = PhysAddr::from_usize(_skernel as usize - KIMAGE_VADDR);
    let end = PhysAddr::from_usize(_ekernel as usize - KIMAGE_VADDR);
    start..end
}
