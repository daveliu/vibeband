use rodio::Source;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const RING_SIZE: usize = 4096;

pub struct RingBuffer {
    buf: Vec<f32>,
    pos: usize,
}

impl RingBuffer {
    fn new() -> Self {
        Self {
            buf: vec![0.0; RING_SIZE],
            pos: 0,
        }
    }

    pub fn push(&mut self, sample: f32) {
        self.buf[self.pos] = sample;
        self.pos = (self.pos + 1) % RING_SIZE;
    }

    pub fn samples(&self, n: usize) -> Vec<f32> {
        let n = n.min(RING_SIZE);
        let mut out = vec![0.0; n];
        let start = (self.pos + RING_SIZE - n) % RING_SIZE;
        for i in 0..n {
            out[i] = self.buf[(start + i) % RING_SIZE];
        }
        out
    }
}

pub type SharedRing = Arc<Mutex<RingBuffer>>;

pub fn new_ring() -> SharedRing {
    Arc::new(Mutex::new(RingBuffer::new()))
}

/// A Source wrapper that copies samples into a shared ring buffer.
pub struct TappedSource<S> {
    inner: S,
    ring: SharedRing,
    channels: u16,
    chan_idx: u16,
    left: f32,
}

impl<S> TappedSource<S>
where
    S: Source<Item = f32>,
{
    pub fn new(inner: S, ring: SharedRing) -> Self {
        let channels = inner.channels();
        Self {
            inner,
            ring,
            channels,
            chan_idx: 0,
            left: 0.0,
        }
    }
}

impl<S> Iterator for TappedSource<S>
where
    S: Source<Item = f32>,
{
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let sample = self.inner.next()?;
        if self.channels == 1 {
            if let Ok(mut ring) = self.ring.lock() {
                ring.push(sample);
            }
        } else {
            // Mix to mono: average L+R
            if self.chan_idx == 0 {
                self.left = sample;
            }
            if self.chan_idx == self.channels - 1 {
                let mono = (self.left + sample) * 0.5;
                if let Ok(mut ring) = self.ring.lock() {
                    ring.push(mono);
                }
            }
        }
        self.chan_idx = (self.chan_idx + 1) % self.channels;
        Some(sample)
    }
}

impl<S> Source for TappedSource<S>
where
    S: Source<Item = f32>,
{
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
    }

    fn channels(&self) -> u16 {
        self.inner.channels()
    }

    fn sample_rate(&self) -> u32 {
        self.inner.sample_rate()
    }

    fn total_duration(&self) -> Option<Duration> {
        self.inner.total_duration()
    }
}
