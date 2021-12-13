use circular_queue::CircularQueue;
use floating_duration::TimeAsFloat;
use std::time::{Duration, Instant};

pub struct Profile {
    start: Option<Instant>,
    end: Option<Instant>,
    duration: Option<Duration>,
    pub duration_buffer: CircularQueue<f32>,
    pub avg_ms: f32,
    pub max_ms: f32,
    pub current_ms: f32,
    pub fps: f32,
}

impl Profile {
    pub fn new(capacity: usize) -> Profile {
        Profile {
            start: None,
            end: None,
            duration: None,
            duration_buffer: CircularQueue::with_capacity(capacity),
            avg_ms: 0.0,
            max_ms: 0.0,
            current_ms: 0.0,
            fps: 0.0,
        }
    }

    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    pub fn end(&mut self) {
        self.end = Some(Instant::now());
        if let (Some(start), Some(end)) = (self.start, self.end) {
            if let Some(duration) = self.duration {
                self.duration = Some(duration + (end - start))
            } else {
                self.duration = Some(end - start);
            }
        }
    }

    pub fn frame(&mut self) {
        // Update buffer
        if let Some(duration) = self.duration {
            let duration_ms = (duration.as_fractional_secs() * 1000.0) as f32;
            self.duration_buffer.push(duration_ms);
        } else {
            self.duration_buffer.push(0.0);
        }
        self.start = None;
        self.end = None;
        self.duration = None;
        // Calculate interesting values
        let mut total_ms = 0.0;
        self.max_ms = 0.0;
        for duration in self.duration_buffer.iter() {
            total_ms = total_ms + *duration;
            if *duration > self.max_ms {
                self.max_ms = *duration;
            }
        }
        self.avg_ms = total_ms / self.duration_buffer.capacity() as f32;
        self.current_ms = if let Some(duration) = self.duration_buffer.iter().next() {
            *duration
        } else {
            0.0
        };
        self.fps = if self.avg_ms > 0.0 {
            1.0 / (self.avg_ms / 1000.0)
        } else {
            0.0
        };
    }
}
