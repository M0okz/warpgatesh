use std::time::Duration;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyncSchedule {
    pub interval: Duration,
    pub jitter: Duration,
    pub initial_backoff: Duration,
    pub maximum_backoff: Duration,
}

impl Default for SyncSchedule {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(5 * 60),
            jitter: Duration::from_secs(30),
            initial_backoff: Duration::from_secs(15),
            maximum_backoff: Duration::from_secs(15 * 60),
        }
    }
}

impl SyncSchedule {
    #[must_use]
    pub fn retry_delay(self, consecutive_failures: u32) -> Duration {
        if consecutive_failures == 0 {
            return Duration::ZERO;
        }

        let exponent = consecutive_failures.saturating_sub(1).min(31);
        let multiplier = 1_u32 << exponent;
        self.initial_backoff
            .saturating_mul(multiplier)
            .min(self.maximum_backoff)
    }

    #[must_use]
    pub fn periodic_delay(self, jitter_seed: u64) -> Duration {
        if self.jitter.is_zero() {
            return self.interval;
        }

        let width = self.jitter.as_secs().saturating_mul(2).saturating_add(1);
        let offset = jitter_seed % width;
        let lower = self.interval.saturating_sub(self.jitter);
        lower.saturating_add(Duration::from_secs(offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_five_minutes_with_small_jitter() {
        let schedule = SyncSchedule::default();
        assert_eq!(schedule.periodic_delay(0), Duration::from_secs(270));
        assert_eq!(schedule.periodic_delay(60), Duration::from_secs(330));
    }

    #[test]
    fn backs_off_exponentially_and_caps_the_delay() {
        let schedule = SyncSchedule::default();
        assert_eq!(schedule.retry_delay(1), Duration::from_secs(15));
        assert_eq!(schedule.retry_delay(3), Duration::from_secs(60));
        assert_eq!(schedule.retry_delay(20), Duration::from_secs(900));
    }
}
