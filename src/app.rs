use crate::audio::AudioEngine;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Loading { done: usize, total: usize },
    Playing,
    Paused,
}

pub struct App {
    pub state: AppState,
    pub engine: AudioEngine,
    pub selected_layer: usize,
    pub should_quit: bool,
    pub errors: Vec<String>,
}

impl App {
    pub fn new(engine: AudioEngine, total_layers: usize) -> Self {
        Self {
            state: AppState::Loading {
                done: 0,
                total: total_layers,
            },
            engine,
            selected_layer: 0,
            should_quit: false,
            errors: Vec::new(),
        }
    }

    pub fn layer_loaded(&mut self) {
        if let AppState::Loading { done, total } = &mut self.state {
            *done += 1;
            if *done >= *total {
                self.state = AppState::Playing;
            }
        }
    }

    pub fn load_error(&mut self, msg: String) {
        self.errors.push(msg);
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if matches!(self.state, AppState::Loading { .. }) {
            if key.code == KeyCode::Char('q') {
                self.should_quit = true;
            }
            return;
        }

        match key.code {
            KeyCode::Char('q') => {
                self.should_quit = true;
            }
            KeyCode::Char(' ') => {
                self.state = match self.state {
                    AppState::Playing => {
                        self.engine.pause_all();
                        AppState::Paused
                    }
                    AppState::Paused => {
                        self.engine.play_all();
                        AppState::Playing
                    }
                    _ => return,
                };
            }
            KeyCode::Tab => {
                if !self.engine.layers.is_empty() {
                    self.selected_layer =
                        (self.selected_layer + 1) % self.engine.layers.len();
                }
            }
            KeyCode::BackTab => {
                if !self.engine.layers.is_empty() {
                    self.selected_layer = if self.selected_layer == 0 {
                        self.engine.layers.len() - 1
                    } else {
                        self.selected_layer - 1
                    };
                }
            }
            KeyCode::Up => {
                let idx = self.selected_layer;
                if let Some(layer) = self.engine.layers.get(idx) {
                    let new_vol = (layer.volume + 0.05).min(1.0);
                    self.engine.set_layer_volume(idx, new_vol);
                }
            }
            KeyCode::Down => {
                let idx = self.selected_layer;
                if let Some(layer) = self.engine.layers.get(idx) {
                    let new_vol = (layer.volume - 0.05).max(0.0);
                    self.engine.set_layer_volume(idx, new_vol);
                }
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                let new_vol = (self.engine.master_volume + 0.05).min(1.0);
                self.engine.set_master_volume(new_vol);
            }
            KeyCode::Char('-') => {
                let new_vol = (self.engine.master_volume - 0.05).max(0.0);
                self.engine.set_master_volume(new_vol);
            }
            _ => {}
        }
    }
}
