use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::Conversation;

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
}

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<Content>,
}

#[derive(Deserialize)]
struct Content {
    text: String,
}

pub async fn produce(transcript: &str, api_key: &str) -> Result<Conversation> {
    let prompt = format!(
        r#"You are given a raw transcript of a conversation. Your task is to convert it into a structured JSON format.

Each line in the transcript follows this pattern: "Speaker: message"

Convert this into a JSON object with the following structure:
{{
    "interjections": [
        {{
            "voice": "Speaker's name",
            "words": "The spoken text without the speaker name"
        }}
    ]
}}

Important:
- Extract the speaker name (everything before the first colon) as the "voice"
- Extract the message (everything after the first colon and space) as the "message"
- Preserve the exact order of messages
- Return ONLY the JSON object, no additional text

Transcript:
{transcript}"#
    );

    let request = AnthropicRequest {
        model: "claude-sonnet-4-20250514".to_string(),
        max_tokens: 16_384,
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&request)
        .send()
        .await
        .context("Failed to send request to Anthropic API")?
        .error_for_status()
        .context("Anthropic API error")?;

    let api_response: AnthropicResponse = response
        .json()
        .await
        .context("Failed to parse Anthropic API response")?;

    let json_text = api_response
        .content
        .first()
        .context("No content in Anthropic response")?
        .text
        .trim()
        .trim_start_matches("```json")
        .trim_end_matches("```")
        .trim();

    let conversation: Conversation = serde_json::from_str(json_text).context(format!(
        "Failed to parse JSON response from Anthropic from: `{json_text}`"
    ))?;

    Ok(conversation)
}
