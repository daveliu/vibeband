use anyhow::{bail, Context, Result};
use serde::Serialize;

const API_URL: &str = "https://api.elevenlabs.io/v1/sound-generation";
const MODEL_ID: &str = "eleven_text_to_sound_v2";
const DURATION: f32 = 30.0;

#[derive(Serialize)]
struct SoundGenRequest {
    text: String,
    duration_seconds: f32,
    #[serde(rename = "loop")]
    loop_audio: bool,
    model_id: String,
}

pub struct ElevenLabsClient {
    client: reqwest::Client,
    api_key: String,
}

impl ElevenLabsClient {
    pub fn new() -> Result<Self> {
        let api_key =
            std::env::var("ELEVENLABS_API_KEY").context("ELEVENLABS_API_KEY env var not set")?;
        Ok(Self {
            client: reqwest::Client::new(),
            api_key,
        })
    }

    pub async fn generate_sound(&self, prompt: &str) -> Result<Vec<u8>> {
        let body = SoundGenRequest {
            text: prompt.to_string(),
            duration_seconds: DURATION,
            loop_audio: true,
            model_id: MODEL_ID.to_string(),
        };

        let resp = self
            .client
            .post(API_URL)
            .header("xi-api-key", &self.api_key)
            .json(&body)
            .send()
            .await
            .context("failed to send request to ElevenLabs")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("ElevenLabs API error {status}: {text}");
        }

        let bytes = resp.bytes().await.context("failed to read response body")?;
        Ok(bytes.to_vec())
    }
}
