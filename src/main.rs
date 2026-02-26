mod api;
mod app;
mod audio;
mod cache;
mod presets;
mod tap;
mod tui;
mod user_presets;
mod visualizer;

use anyhow::{bail, Result};
use app::App;
use audio::AudioEngine;
use clap::Parser;
use crossterm::{
    event::{self, Event, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use presets::PRESETS;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::stdout;
use std::time::Duration;
use tokio::sync::mpsc;
use visualizer::Visualizer;

/// Ambient sound generator for your terminal
#[derive(Parser)]
#[command(name = "vibeband", version, about)]
struct Cli {
    /// Scene name(s) to play, or a subcommand: save, remove, list.
    scenes: Vec<String>,

    /// Custom prompt(s) — generate any sound you describe.
    #[arg(short, long = "custom", value_name = "PROMPT")]
    custom: Vec<String>,
}

struct Scene {
    name: String,
    emoji: String,
    label: String,
    prompt: String,
}

struct LoadedLayer {
    name: String,
    emoji: String,
    label: String,
    result: Result<Vec<u8>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle subcommands: save, remove, list
    if let Some(first) = cli.scenes.first() {
        match first.as_str() {
            "save" => return cmd_save(&cli.scenes[1..]),
            "remove" => return cmd_remove(&cli.scenes[1..]),
            "list" => return cmd_list(),
            _ => {}
        }
    }

    if cli.scenes.is_empty() && cli.custom.is_empty() {
        print_help()?;
        return Ok(());
    }

    let mut scenes: Vec<Scene> = Vec::new();

    // Collect preset + user preset scenes
    let scene_names: Vec<&str> = cli
        .scenes
        .iter()
        .map(|s| s.as_str())
        .filter(|s| *s != "mix")
        .collect();

    for name in &scene_names {
        if let Some(p) = presets::find_preset(name) {
            scenes.push(Scene {
                name: p.name.to_string(),
                emoji: p.emoji.to_string(),
                label: p.label.to_string(),
                prompt: p.prompt.to_string(),
            });
        } else if let Ok(Some(up)) = user_presets::find(name) {
            scenes.push(Scene {
                name: name.to_string(),
                emoji: up.emoji.clone(),
                label: up.label.clone(),
                prompt: up.prompt.clone(),
            });
        } else {
            // Collect all available names
            let mut available: Vec<String> =
                PRESETS.iter().map(|p| p.name.to_string()).collect();
            if let Ok(user) = user_presets::load() {
                for key in user.presets.keys() {
                    available.push(key.clone());
                }
            }
            bail!(
                "Unknown scene: '{}'. Available: {}",
                name,
                available.join(", ")
            );
        }
    }

    // Collect custom prompts
    for (i, prompt) in cli.custom.iter().enumerate() {
        let label: String = prompt
            .split_whitespace()
            .take(3)
            .collect::<Vec<&str>>()
            .join(" ");
        let label = if label.len() < prompt.len() {
            format!("{}…", label)
        } else {
            label
        };

        scenes.push(Scene {
            name: format!("custom_{}", i + 1),
            emoji: "🎵".to_string(),
            label,
            prompt: prompt.clone(),
        });
    }

    if scenes.is_empty() {
        bail!("No scenes specified. Run `vibeband` to see available options.");
    }

    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal, scenes).await;

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    result
}

fn print_help() -> Result<()> {
    println!("vibeband — ambient sounds for your terminal\n");
    println!("Play:");
    println!("  vibeband <scene>                        Play a preset scene");
    println!("  vibeband mix <scene> <scene>             Mix multiple scenes");
    println!("  vibeband -c \"your prompt here\"           Play a custom sound");
    println!("  vibeband cafe -c \"vinyl crackle\"         Mix preset + custom\n");
    println!("Manage custom presets:");
    println!("  vibeband save <name> \"<prompt>\"          Save/update a preset");
    println!("  vibeband remove <name>                   Remove a custom preset");
    println!("  vibeband list                            List all presets\n");
    println!("Built-in presets:");
    for p in PRESETS {
        println!("  {} {:<12} {}", p.emoji, p.name, p.label);
    }

    // Show user presets
    if let Ok(user) = user_presets::load() {
        if !user.presets.is_empty() {
            println!("\nYour presets:");
            for (name, p) in &user.presets {
                println!("  {} {:<12} {}", p.emoji, name, p.label);
            }
        }
    }

    println!("\nExamples:");
    println!("  vibeband save myguitar \"soft acoustic guitar fingerpicking with reverb\"");
    println!("  vibeband myguitar");
    println!("  vibeband mix rain myguitar");
    Ok(())
}

fn cmd_save(args: &[String]) -> Result<()> {
    if args.len() < 2 {
        bail!("Usage: vibeband save <name> \"<prompt>\"\n\nExample:\n  vibeband save myguitar \"soft acoustic guitar fingerpicking with reverb\"");
    }

    let name = &args[0];
    let prompt = args[1..].join(" ");

    // Don't allow overwriting built-in presets
    if presets::find_preset(name).is_some() {
        bail!(
            "Cannot overwrite built-in preset '{}'. Choose a different name.",
            name
        );
    }

    // Generate a label from first 3 words
    let label: String = prompt
        .split_whitespace()
        .take(3)
        .collect::<Vec<&str>>()
        .join(" ");
    let label = if label.len() < prompt.len() {
        format!("{}…", label)
    } else {
        label
    };

    let is_update = user_presets::find(name)?.is_some();
    user_presets::save(name, "🎵", &label, &prompt)?;

    if is_update {
        println!("Updated preset '{}': {}", name, prompt);
    } else {
        println!("Saved preset '{}': {}", name, prompt);
    }
    println!("\nUse it with: vibeband {}", name);
    Ok(())
}

fn cmd_remove(args: &[String]) -> Result<()> {
    if args.is_empty() {
        bail!("Usage: vibeband remove <name>");
    }

    let name = &args[0];

    if presets::find_preset(name).is_some() {
        bail!("Cannot remove built-in preset '{}'.", name);
    }

    if user_presets::remove(name)? {
        println!("Removed preset '{}'.", name);
    } else {
        bail!("No custom preset named '{}'.", name);
    }
    Ok(())
}

fn cmd_list() -> Result<()> {
    println!("Built-in presets:");
    for p in PRESETS {
        println!("  {} {:<12} {}", p.emoji, p.name, p.label);
    }

    let user = user_presets::load()?;
    if user.presets.is_empty() {
        println!("\nNo custom presets yet. Save one with:");
        println!("  vibeband save <name> \"<prompt>\"");
    } else {
        println!("\nYour presets:");
        for (name, p) in &user.presets {
            println!("  {} {:<12} {} — {}", p.emoji, name, p.label, p.prompt);
        }
    }
    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    scenes: Vec<Scene>,
) -> Result<()> {
    let engine = AudioEngine::new()?;
    let total = scenes.len();
    let mut app = App::new(engine, total);
    let mut vis = Visualizer::new(44100.0);

    let (tx, mut rx) = mpsc::channel::<LoadedLayer>(total);

    let client = api::ElevenLabsClient::new()?;
    let client = std::sync::Arc::new(client);

    for scene in &scenes {
        let tx = tx.clone();
        let client = client.clone();
        let prompt = scene.prompt.clone();
        let name = scene.name.clone();
        let emoji = scene.emoji.clone();
        let label = scene.label.clone();

        tokio::spawn(async move {
            let result = match cache::read_cache(&prompt) {
                Ok(Some(cached)) => Ok(cached),
                _ => match client.generate_sound(&prompt).await {
                    Ok(generated) => {
                        let _ = cache::write_cache(&prompt, &generated);
                        Ok(generated)
                    }
                    Err(e) => Err(e),
                },
            };
            let _ = tx
                .send(LoadedLayer {
                    name,
                    emoji,
                    label,
                    result,
                })
                .await;
        });
    }
    drop(tx);

    let tick_rate = Duration::from_millis(50);
    let mut loading_done = false;

    loop {
        terminal.draw(|f| tui::draw(f, &app, &mut vis))?;

        while event::poll(Duration::ZERO)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.on_key(key);
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }

        if !loading_done {
            match rx.try_recv() {
                Ok(layer) => match layer.result {
                    Ok(data) => {
                        app.engine
                            .add_layer(&layer.name, &layer.emoji, &layer.label, data)?;
                        if let Some(last) = app.engine.layers.last() {
                            vis.add_ring(last.ring.clone());
                        }
                        app.layer_loaded();
                    }
                    Err(e) => {
                        app.load_error(format!("{}: {}", layer.label, e));
                        app.layer_loaded();
                    }
                },
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    loading_done = true;
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
            }
        }

        tokio::time::sleep(tick_rate).await;
    }
}
