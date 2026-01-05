use core::cell::Cell;
use core::task::Waker;
use cortex_m::interrupt::Mutex;
use cortex_m::peripheral::SYST;
use cortex_m_rt::exception;
use embassy_time_driver::Driver;

struct SystickDriver;

embassy_time_driver::time_driver_impl!(static DRIVER: SystickDriver = SystickDriver);

impl Driver for SystickDriver {
    fn now(&self) -> u64 {
        cortex_m::interrupt::free(|cs| TICKS.borrow(cs).get())
    }

    fn schedule_wake(&self, _at: u64, _waker: &Waker) {
        // We rely on the periodic SysTick interrupt to wake the executor.
        // The executor (WFE) will wake up every 1ms, check the time, and run tasks if ready.
        // This is sufficient for a QEMU demo.

        // IMPORTANT: In QEMU, WFE might sleep too deeply or not wake up correctly if we don't
        // explicitly trigger an interrupt or if the SysTick is the only source.
        // However, since SysTick is running, it should be fine.
        // But to be safe, we can wake the waker.
        _waker.wake_by_ref();
    }
}

static TICKS: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));

#[exception]
fn SysTick() {
    cortex_m::interrupt::free(|cs| {
        let ticks = TICKS.borrow(cs);
        let t = ticks.get() + 1;
        ticks.set(t);
    });
}

pub fn init(syst: &mut SYST) {
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    // MPS2-AN386 runs at 25MHz. 25_000_000 / 1000 = 25_000
    syst.set_reload(25_000 - 1);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();
}
