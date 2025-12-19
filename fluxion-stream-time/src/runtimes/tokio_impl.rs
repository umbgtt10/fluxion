#[cfg(feature = "time-tokio")]
pub mod implementation {
    use crate::timer::Timer;
    use std::time::{Duration, Instant};
    use tokio::time::sleep;

    #[derive(Clone, Debug)]
    pub struct TokioTimer;

    impl Timer for TokioTimer {
        type Sleep = tokio::time::Sleep;

        type Instant = Instant;

        fn sleep_future(&self, duration: Duration) -> Self::Sleep {
            sleep(duration)
        }

        fn now(&self) -> Self::Instant {
            Instant::now()
        }
    }
}
