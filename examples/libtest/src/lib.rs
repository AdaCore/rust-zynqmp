use adacore_zynqmp as _;

#[cfg(test)]
mod tests {
    // These helpers read the hardware counter directly so the regression
    // test below can use them as ground truth, independent of the
    // `cntvct`/`cntfrq` accessors in `adacore-zynqmp`'s newlib PAL.
    fn hw_counter_freq() -> u64 {
        let freq: u64;
        unsafe { core::arch::asm!("mrs {0}, cntfrq_el0", out(reg) freq) }
        freq
    }

    fn hw_counter_ticks() -> u64 {
        let value: u64;
        unsafe { core::arch::asm!("mrs {0}, cntvct_el0", out(reg) value) }
        value
    }

    fn spin(ticks: u64) -> u64 {
        let start = hw_counter_ticks();
        let target = start + ticks;
        while hw_counter_ticks() < target {}
        hw_counter_ticks() - start
    }

    #[test]
    fn vec_grows_across_reallocations() {
        let mut values = Vec::new();
        for value in 0..1000 {
            values.push(value);
        }
        assert_eq!(values.len(), 1000);
        assert_eq!(values[500], 500);
    }

    #[test]
    fn hashmap_insert_and_lookup() {
        use std::collections::HashMap;
        let mut map = HashMap::new();
        map.insert("hello", 1);
        map.insert("world", 2);
        assert_eq!(map.get("hello"), Some(&1));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn instant_measures_elapsed_time() {
        let start = std::time::Instant::now();
        let _: Vec<u32> = (0..10_000).collect();
        assert!(start.elapsed() > std::time::Duration::ZERO);
    }

    #[test]
    fn instant_elapsed_matches_hardware_counter() {
        use std::time::{Duration, Instant};

        let freq = hw_counter_freq();
        let start = Instant::now();
        let elapsed_ticks = spin(freq);
        let elapsed = start.elapsed();

        let expected = Duration::from_nanos(elapsed_ticks * 1_000_000_000 / freq);
        let tolerance = Duration::from_millis(100);
        let diff = if elapsed > expected {
            elapsed - expected
        } else {
            expected - elapsed
        };
        assert!(
            diff <= tolerance,
            "Instant::elapsed() was {:?}, hardware counter says ~{:?} (difference {:?}, tolerance {:?})",
            elapsed,
            expected,
            diff,
            tolerance,
        );
    }
}
