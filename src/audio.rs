use crate::transcript::Interjection;
use anyhow::{Context, Result};
use futures_util::StreamExt;
use serde::Serialize;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Serialize)]
struct ElevenLabsRequest {
    text: String,
    model_id: String,
    voice_settings: VoiceSettings,
}

#[derive(Debug, Serialize)]
struct VoiceSettings {
    stability: f32,
    similarity_boost: f32,
}

pub async fn generate_audio(
    interjection: &Interjection,
    voice_id: &str,
    file_path: &Path,
) -> Result<()> {
    let api_key = std::env::var("ELEVENLABS_API_KEY")
        .context("ELEVENLABS_API_KEY environment variable not set")?;

    let client = reqwest::Client::new();

    let request_body = ElevenLabsRequest {
        text: interjection.words.clone(),
        model_id: "eleven_multilingual_v2".to_string(),
        voice_settings: VoiceSettings {
            stability: 0.5,
            similarity_boost: 0.5,
        },
    };

    let response = client
        .post(format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{}",
            voice_id
        ))
        .header("xi-api-key", &api_key)
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .context("Failed to send request to ElevenLabs API")?
        .error_for_status()
        .context("ElevenLabs API error")?;

    let mut file = File::create(file_path)
        .await
        .context("Failed to create output file")?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.context("Failed to read chunk from response")?;
        file.write_all(&chunk)
            .await
            .context("Failed to write chunk to file")?;
    }

    file.flush().await.context("Failed to flush file")?;

    Ok(())
}
