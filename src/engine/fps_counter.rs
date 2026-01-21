//! FPS counter and timing utilities.
//!
//! Tracks frame timing for performance monitoring.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Tracks frame timing and calculates FPS.
#[derive(Debug)]
pub struct FpsCounter {
    /// Timestamps of recent frames.
    frame_times: VecDeque<Instant>,
    /// Last time FPS was logged.
    last_log: Instant,
    /// How often to log FPS (if logging is enabled).
    log_interval: Duration,
    /// Maximum number of frames to track for averaging.
    max_samples: usize,
}

impl Default for FpsCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl FpsCounter {
    /// Creates a new FPS counter.
    #[must_use]
    pub fn new() -> Self {
        Self {
            frame_times: VecDeque::with_capacity(120),
            last_log: Instant::now(),
            log_interval: Duration::from_secs(1),
            max_samples: 100,
        }
    }

    /// Records a new frame and returns the current FPS.
    pub fn tick(&mut self) -> f64 {
        let now = Instant::now();
        self.frame_times.push_back(now);

        // Remove old samples
        while self.frame_times.len() > self.max_samples {
            self.frame_times.pop_front();
        }

        self.fps()
    }

    /// Returns the current FPS based on recent frame times.
    #[must_use]
    pub fn fps(&self) -> f64 {
        if self.frame_times.len() < 2 {
            return 0.0;
        }

        let oldest = self.frame_times.front().unwrap();
        let newest = self.frame_times.back().unwrap();
        let duration = newest.duration_since(*oldest);

        if duration.is_zero() {
            return 0.0;
        }

        (self.frame_times.len() - 1) as f64 / duration.as_secs_f64()
    }

    /// Returns the average frame time in milliseconds.
    #[must_use]
    pub fn frame_time_ms(&self) -> f64 {
        let fps = self.fps();
        if fps > 0.0 { 1000.0 / fps } else { 0.0 }
    }

    /// Checks if it's time to log FPS and returns the value if so.
    ///
    /// Returns `Some(fps)` if the log interval has elapsed, `None` otherwise.
    pub fn should_log(&mut self) -> Option<f64> {
        let now = Instant::now();
        if now.duration_since(self.last_log) >= self.log_interval {
            self.last_log = now;
            Some(self.fps())
        } else {
            None
        }
    }

    /// Sets the log interval.
    pub fn set_log_interval(&mut self, interval: Duration) {
        self.log_interval = interval;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn fps_counter_starts_at_zero() {
        let counter = FpsCounter::new();
        assert_eq!(counter.fps(), 0.0);
    }

    #[test]
    fn fps_counter_tracks_frames() {
        let mut counter = FpsCounter::new();

        // Simulate some frames
        for _ in 0..10 {
            counter.tick();
            thread::sleep(Duration::from_millis(10));
        }

        // Should have some reasonable FPS (around 100 for 10ms frames)
        let fps = counter.fps();
        assert!(fps > 50.0 && fps < 150.0, "FPS was {fps}");
    }

    #[test]
    fn frame_time_ms_is_inverse_of_fps() {
        let mut counter = FpsCounter::new();

        // Simulate 100 FPS (10ms per frame)
        for _ in 0..10 {
            counter.tick();
            thread::sleep(Duration::from_millis(10));
        }

        let fps = counter.fps();
        let frame_time = counter.frame_time_ms();

        // frame_time should be ~1000/fps
        if fps > 0.0 {
            let expected = 1000.0 / fps;
            assert!((frame_time - expected).abs() < 0.1);
        }
    }

    #[test]
    fn frame_time_zero_when_no_fps() {
        let counter = FpsCounter::new();
        assert_eq!(counter.frame_time_ms(), 0.0);
    }

    #[test]
    fn set_log_interval_works() {
        let mut counter = FpsCounter::new();
        counter.set_log_interval(Duration::from_millis(500));
        // Just verify it doesn't panic
    }

    #[test]
    fn should_log_respects_interval() {
        let mut counter = FpsCounter::new();
        counter.set_log_interval(Duration::from_millis(50));

        // First call right after creation shouldn't log
        assert!(counter.should_log().is_none());

        // Wait for interval
        thread::sleep(Duration::from_millis(60));

        // Now it should log
        assert!(counter.should_log().is_some());

        // Immediately after, it shouldn't log again
        assert!(counter.should_log().is_none());
    }

    #[test]
    fn tick_returns_current_fps() {
        let mut counter = FpsCounter::new();

        for _ in 0..5 {
            let fps = counter.tick();
            thread::sleep(Duration::from_millis(5));
            // FPS should be non-negative
            assert!(fps >= 0.0);
        }
    }

    #[test]
    fn default_matches_new() {
        let counter1 = FpsCounter::new();
        let counter2 = FpsCounter::default();
        assert_eq!(counter1.fps(), counter2.fps());
    }
}
