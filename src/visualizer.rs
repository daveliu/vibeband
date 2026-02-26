use crate::tap::SharedRing;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};
use rustfft::{num_complex::Complex, Fft, FftPlanner};
use std::sync::Arc;

const FFT_SIZE: usize = 2048;
const NUM_BANDS: usize = 10;
const BAR_BLOCKS: [&str; 9] = [" ", "▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];

// Frequency edges for 10 spectrum bands (Hz)
const BAND_EDGES: [f64; 11] = [
    20.0, 100.0, 200.0, 400.0, 800.0, 1600.0, 3200.0, 6400.0, 12800.0, 16000.0, 20000.0,
];

const COLOR_LOW: Color = Color::Green;
const COLOR_MID: Color = Color::Yellow;
const COLOR_HIGH: Color = Color::Red;

pub struct Visualizer {
    rings: Vec<SharedRing>,
    prev: [f64; NUM_BANDS],
    sample_rate: f64,
    fft: Arc<dyn Fft<f64>>,
    fft_buf: Vec<Complex<f64>>,
}

impl Visualizer {
    pub fn new(sample_rate: f64) -> Self {
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(FFT_SIZE);
        Self {
            rings: Vec::new(),
            prev: [0.0; NUM_BANDS],
            sample_rate,
            fft,
            fft_buf: vec![Complex::new(0.0, 0.0); FFT_SIZE],
        }
    }

    pub fn add_ring(&mut self, ring: SharedRing) {
        self.rings.push(ring);
    }

    pub fn analyze(&mut self) -> [f64; NUM_BANDS] {
        let mut bands = [0.0; NUM_BANDS];

        if self.rings.is_empty() {
            self.decay();
            return self.prev;
        }

        // Mix samples from all rings
        let mut mixed = vec![0.0f32; FFT_SIZE];
        let mut active = 0;
        for ring in &self.rings {
            if let Ok(ring) = ring.lock() {
                let samples = ring.samples(FFT_SIZE);
                for (i, s) in samples.iter().enumerate() {
                    mixed[i] += s;
                }
                active += 1;
            }
        }

        if active == 0 {
            self.decay();
            return self.prev;
        }

        // Average
        let scale = 1.0 / active as f32;
        for s in &mut mixed {
            *s *= scale;
        }

        // Apply Hann window and fill FFT buffer
        for (i, &s) in mixed.iter().enumerate() {
            let w = 0.5
                * (1.0
                    - (2.0 * std::f64::consts::PI * i as f64 / (FFT_SIZE - 1) as f64).cos());
            self.fft_buf[i] = Complex::new(s as f64 * w, 0.0);
        }

        // FFT
        self.fft.process(&mut self.fft_buf);

        let bin_hz = self.sample_rate / FFT_SIZE as f64;
        let half_len = FFT_SIZE / 2;

        // Sum raw magnitudes per frequency band — no 2/N scaling,
        // matching cliamp's approach for noise-like ambient audio
        for b in 0..NUM_BANDS {
            let lo_idx = (BAND_EDGES[b] / bin_hz) as usize;
            let hi_idx = (BAND_EDGES[b + 1] / bin_hz) as usize;
            let lo_idx = lo_idx.max(1);
            let hi_idx = hi_idx.min(half_len - 1);

            let mut sum = 0.0;
            let mut count = 0;
            for i in lo_idx..=hi_idx {
                sum += self.fft_buf[i].norm();
                count += 1;
            }
            if count > 0 {
                sum /= count as f64;
            }

            // dB scale matching cliamp: (20*log10(mag) + 10) / 50
            if sum > 0.0 {
                bands[b] = (20.0 * sum.log10() + 10.0) / 50.0;
            }
            bands[b] = bands[b].clamp(0.0, 1.0);

            // Temporal smoothing: fast attack, slow decay
            if bands[b] > self.prev[b] {
                bands[b] = bands[b] * 0.6 + self.prev[b] * 0.4;
            } else {
                bands[b] = bands[b] * 0.25 + self.prev[b] * 0.75;
            }
            self.prev[b] = bands[b];
        }

        bands
    }

    fn decay(&mut self) {
        for b in 0..NUM_BANDS {
            self.prev[b] *= 0.8;
        }
    }

    pub fn render(&mut self, width: usize, height: usize) -> Vec<Line<'static>> {
        let bands = self.analyze();
        let bar_width = ((width + 1) / (NUM_BANDS + 1)).max(1);
        let bar_height = height.max(2);

        let mut lines = Vec::with_capacity(bar_height);

        for row in 0..bar_height {
            let row_bottom = (bar_height - 1 - row) as f64 / bar_height as f64;
            let row_top = (bar_height - row) as f64 / bar_height as f64;

            let mut spans = Vec::new();

            for (i, &level) in bands.iter().enumerate() {
                let block = if level >= row_top {
                    "█"
                } else if level > row_bottom {
                    let frac = (level - row_bottom) / (row_top - row_bottom);
                    let idx = (frac * (BAR_BLOCKS.len() - 1) as f64) as usize;
                    BAR_BLOCKS[idx.min(BAR_BLOCKS.len() - 1)]
                } else {
                    " "
                };

                let color = if row_bottom >= 0.6 {
                    COLOR_HIGH
                } else if row_bottom >= 0.3 {
                    COLOR_MID
                } else {
                    COLOR_LOW
                };

                let bar_str: String = block.repeat(bar_width);
                spans.push(Span::styled(bar_str, Style::default().fg(color)));

                if i < NUM_BANDS - 1 {
                    spans.push(Span::raw(" "));
                }
            }

            lines.push(Line::from(spans));
        }

        lines
    }
}
