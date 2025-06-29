use anyhow::Result;

use super::{Conversation, Interjection};

pub async fn produce(transcript: &str) -> Result<Conversation> {
    Ok(Conversation {
        interjections: transcript
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .map(|line| {
                let (voice, words) = line.split_once(": ").ok_or(anyhow::format_err!(
                    "Line does not start with a voice labelling: `{line}`"
                ))?;

                Ok(Interjection {
                    voice: voice.to_owned(),
                    words: words.trim().to_owned(),
                })
            })
            .collect::<Result<Vec<_>>>()?,
    })
}
