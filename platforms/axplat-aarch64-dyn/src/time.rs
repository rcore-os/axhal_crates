use aarch64_cpu::registers::*;
use axplat::time::TimeIf;
use lazyinit::LazyInit;
#[cfg(not(feature = "irq"))]
use rdrive::driver::intc::Trigger;
use rdrive::{IrqConfig, PlatformDevice, module_driver, probe::OnProbeError, register::FdtInfo};

static TIMER_IRQ_CONFIG: LazyInit<IrqConfig> = LazyInit::new();

struct TimeIfImpl;

#[impl_plat_interface]
impl TimeIf for TimeIfImpl {
    /// Returns the current clock time in hardware ticks.
    fn current_ticks() -> u64 {
        CNTPCT_EL0.get()
    }

    /// Converts hardware ticks to nanoseconds.
    fn ticks_to_nanos(ticks: u64) -> u64 {
        let freq = CNTFRQ_EL0.get();
        // Convert ticks to nanoseconds using the frequency.
        (ticks * axplat::time::NANOS_PER_SEC) / freq
    }

    /// Converts nanoseconds to hardware ticks.
    fn nanos_to_ticks(nanos: u64) -> u64 {
        let freq = CNTFRQ_EL0.get();
        // Convert nanoseconds to ticks using the frequency.
        (nanos * freq) / axplat::time::NANOS_PER_SEC
    }

    /// Return epoch offset in nanoseconds (wall time offset to monotonic
    /// clock start).
    fn epochoffset_nanos() -> u64 {
        0
    }

    /// Set a one-shot timer.
    ///
    /// A timer interrupt will be triggered at the specified monotonic time
    /// deadline (in nanoseconds).
    #[cfg(feature = "irq")]
    fn set_oneshot_timer(deadline_ns: u64) {
        let cnptct = CNTPCT_EL0.get();
        let cnptct_deadline = Self::nanos_to_ticks(deadline_ns);
        if cnptct < cnptct_deadline {
            let interval = cnptct_deadline - cnptct;
            debug_assert!(interval <= u32::MAX as u64);
            CNTP_TVAL_EL0.set(interval);
        } else {
            CNTP_TVAL_EL0.set(0);
        }
    }

    #[cfg(feature = "irq")]
    fn irq_num() -> usize {
        TIMER_IRQ_CONFIG.irq.into()
    }
}

#[cfg(feature = "irq")]
/// Enable timer interrupts.
///
/// It should be called on all CPUs, as the timer interrupt is a PPI (Private
/// Peripheral Interrupt).
pub fn enable_irqs() {
    CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET);
    CNTP_TVAL_EL0.set(0);
    crate::irq::set_enable(
        TIMER_IRQ_CONFIG.irq.into(),
        true,
        Some(TIMER_IRQ_CONFIG.trigger),
        false,
    );
}

module_driver!(
    name: "ARMv8 Timer",
    level: ProbeLevel::PreKernel,
    priority: ProbePriority::DEFAULT,
    probe_kinds: &[
        ProbeKind::Fdt {
            compatibles: &["arm,armv8-timer"],
            on_probe: probe
        }
    ],
);

fn probe(_fdt: FdtInfo<'_>, _dev: PlatformDevice) -> Result<(), OnProbeError> {
    #[cfg(not(feature = "irq"))]
    let irq = IrqConfig {
        irq: 0.into(),
        trigger: Trigger::EdgeBoth,
        is_private: true,
    };
    #[cfg(feature = "irq")]
    let irq = {
        #[cfg(not(feature = "hv"))]
        let irq_idx = 1;
        #[cfg(feature = "hv")]
        let irq_idx = 3;
        _dev.descriptor.irqs[irq_idx].clone()
    };
    TIMER_IRQ_CONFIG.call_once(|| irq);
    Ok(())
}
