mod audio;
mod cache;
mod transcript;

use anyhow::Context;
use serde::Deserialize;
use std::io::Read;
use std::path::PathBuf;
use std::{collections::HashMap, os::unix::fs::MetadataExt};
use structopt::StructOpt;
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, EnumString, Display)]
#[strum(serialize_all = "lowercase")]
enum TranscriptionMode {
    Anthropic,
    Naive,
}

#[derive(Deserialize, Debug)]
struct Environment {
    anthropic_api_key: String,
}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "conversation-transcript-to-audio",
    about = "Convert conversation transcripts to audio files"
)]
struct Args {
    /// Prefix for output audio files
    #[structopt(short, long)]
    output_prefix: PathBuf,

    /// Path to voice configuration TOML file
    #[structopt(short = "c", long)]
    config: PathBuf,

    /// Transcription mode (anthropic or naive)
    #[structopt(short = "m", long, default_value = "naive")]
    mode: TranscriptionMode,
}

#[derive(Deserialize)]
struct Config {
    voices: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    let environment = envy::from_env::<Environment>()?;
    let args = Args::from_args();

    let config: Config = toml::from_str(
        &tokio::fs::read_to_string(&args.config)
            .await
            .context("Failed to read voice config file")?,
    )
    .context("Failed to parse voice config TOML")?;

    let mut transcript_data = Vec::new();
    std::io::stdin().read_to_end(&mut transcript_data)?;

    let transcript_cache = cache::TranscriptCache::new().await?;
    let cache_key = cache::TranscriptCacheKey {
        transcript_hash: cache::compute_transcript_hash(&transcript_data),
    };

    let conversation = if let Some(cached_conversation) = transcript_cache.get(&cache_key).await {
        println!("Using cached transcription");
        cached_conversation
    } else {
        println!("Transcribing…");
        let conversation = match args.mode {
            TranscriptionMode::Anthropic => {
                transcript::anthropic::produce(
                    std::str::from_utf8(&transcript_data)?,
                    &environment.anthropic_api_key,
                )
                .await?
            }
            TranscriptionMode::Naive => {
                transcript::naive::produce(std::str::from_utf8(&transcript_data)?).await?
            }
        };
        println!("Transcribed.");

        transcript_cache
            .insert(cache_key, conversation.clone())
            .await;
        conversation
    };

    tokio::fs::create_dir_all(&args.output_prefix).await?;

    let total_interjections = conversation.interjections.len();
    let padding_width = total_interjections.to_string().len();

    println!("Generating audio…");

    for (index, interjection) in conversation.interjections.iter().enumerate() {
        let index = index + 1;
        let file_path =
            args.output_prefix
                .join(format!("{:0width$}.mp3", index, width = padding_width));

        if file_path
            .metadata()
            .is_ok_and(|metadata| metadata.size() > 0)
        {
            println!("{} exists, skipping.", file_path.display());
            continue;
        }

        let voice_id = config.voices.get(&interjection.voice).ok_or_else(|| {
            anyhow::anyhow!(
                "No voice ID found for voice '{}' in config file",
                interjection.voice
            )
        })?;

        println!(
            "Generating audio for utterance {index}/{total_interjections}: (voice: {})…",
            interjection.voice
        );

        audio::generate_audio(interjection, voice_id, &file_path).await?;
    }

    println!(
        "Successfully generated {} audio files.",
        total_interjections
    );

    Ok(())
}
