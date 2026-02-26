use crate::tap::{self, SharedRing, TappedSource};
use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use std::io::Cursor;

#[allow(dead_code)]
pub struct Layer {
    pub name: String,
    pub emoji: String,
    pub label: String,
    pub volume: f32,
    pub ring: SharedRing,
    sink: Sink,
}

impl Layer {
    fn update_volume(&self, master: f32) {
        self.sink.set_volume(master * self.volume);
    }
}

pub struct AudioEngine {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    pub layers: Vec<Layer>,
    pub master_volume: f32,
}

impl AudioEngine {
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) =
            OutputStream::try_default().context("failed to open audio output")?;
        Ok(Self {
            _stream: stream,
            stream_handle,
            layers: Vec::new(),
            master_volume: 0.7,
        })
    }

    pub fn add_layer(&mut self, name: &str, emoji: &str, label: &str, audio_data: Vec<u8>) -> Result<()> {
        let sink = Sink::try_new(&self.stream_handle).context("failed to create audio sink")?;
        let cursor = Cursor::new(audio_data);
        let source = Decoder::new(cursor).context("failed to decode audio")?;

        let ring = tap::new_ring();
        let float_source = source.convert_samples::<f32>().repeat_infinite();
        let tapped = TappedSource::new(float_source, ring.clone());
        sink.append(tapped);

        let layer = Layer {
            name: name.to_string(),
            emoji: emoji.to_string(),
            label: label.to_string(),
            volume: 0.8,
            ring,
            sink,
        };
        layer.update_volume(self.master_volume);
        self.layers.push(layer);
        Ok(())
    }

    pub fn pause_all(&self) {
        for layer in &self.layers {
            layer.sink.pause();
        }
    }

    pub fn play_all(&self) {
        for layer in &self.layers {
            layer.sink.play();
        }
    }

    pub fn set_layer_volume(&mut self, index: usize, volume: f32) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.volume = volume.clamp(0.0, 1.0);
            layer.update_volume(self.master_volume);
        }
    }

    pub fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
        for layer in &self.layers {
            layer.update_volume(self.master_volume);
        }
    }
}
