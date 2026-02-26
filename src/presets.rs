pub struct Preset {
    pub name: &'static str,
    pub emoji: &'static str,
    pub label: &'static str,
    pub prompt: &'static str,
}

pub const PRESETS: &[Preset] = &[
    Preset {
        name: "cafe",
        emoji: "☕",
        label: "Coffee Shop",
        prompt: "Busy coffee shop ambiance with espresso machine sounds, gentle chatter, cups clinking, and soft background music",
    },
    Preset {
        name: "rain",
        emoji: "🌧",
        label: "Rain",
        prompt: "Steady rain falling on a rooftop with occasional distant thunder and water dripping",
    },
    Preset {
        name: "forest",
        emoji: "🌲",
        label: "Forest",
        prompt: "Peaceful forest ambiance with birds singing, leaves rustling in gentle wind, and a distant stream",
    },
    Preset {
        name: "fire",
        emoji: "🔥",
        label: "Fireplace",
        prompt: "Cozy crackling fireplace with wood popping and warm ambient room tone",
    },
    Preset {
        name: "ocean",
        emoji: "🌊",
        label: "Ocean",
        prompt: "Ocean waves gently crashing on a beach with seagulls in the distance",
    },
    // Nature
    Preset {
        name: "thunder",
        emoji: "⛈",
        label: "Thunderstorm",
        prompt: "Rolling thunderstorm with heavy rain, deep rumbling thunder, and occasional lightning cracks",
    },
    Preset {
        name: "wind",
        emoji: "💨",
        label: "Wind",
        prompt: "Howling wind blowing through open fields with gusts and whistling",
    },
    Preset {
        name: "creek",
        emoji: "🏞",
        label: "Creek",
        prompt: "Gentle babbling creek with water flowing over rocks and pebbles in a quiet forest",
    },
    Preset {
        name: "birds",
        emoji: "🐦",
        label: "Birdsong",
        prompt: "Morning birdsong chorus with various songbirds singing in a garden at dawn",
    },
    Preset {
        name: "night",
        emoji: "🌙",
        label: "Night",
        prompt: "Nighttime ambiance with crickets chirping, gentle breeze, and occasional owl hooting",
    },
    // Urban/indoor
    Preset {
        name: "train",
        emoji: "🚂",
        label: "Train",
        prompt: "Rhythmic train on tracks with steady clacking, gentle swaying, and distant whistle",
    },
    Preset {
        name: "traffic",
        emoji: "🚗",
        label: "City Traffic",
        prompt: "Urban city traffic ambiance with cars passing, distant horns, and general city hum",
    },
    Preset {
        name: "library",
        emoji: "📚",
        label: "Library",
        prompt: "Quiet library ambiance with soft page turning, distant whispers, and gentle air conditioning hum",
    },
    Preset {
        name: "keyboard",
        emoji: "⌨",
        label: "Keyboard",
        prompt: "Mechanical keyboard typing sounds with rhythmic key presses and spacebar clicks",
    },
    // Musical/textural
    Preset {
        name: "synth",
        emoji: "🎹",
        label: "Synth Pad",
        prompt: "Deep atmospheric synth pad drone with slow modulation and warm ambient texture",
    },
    Preset {
        name: "guitar",
        emoji: "🎸",
        label: "Ambient Guitar",
        prompt: "Soft ambient clean guitar loop with reverb and gentle fingerpicking",
    },
    Preset {
        name: "drums",
        emoji: "🥁",
        label: "Lo-fi Drums",
        prompt: "Lo-fi hip-hop drum loop with dusty vinyl crackle and relaxed boom-bap beat at 85 BPM",
    },
];

pub fn find_preset(name: &str) -> Option<&'static Preset> {
    PRESETS.iter().find(|p| p.name == name)
}
