pub(crate) mod anthropic;
pub(crate) mod naive;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Conversation {
    pub interjections: Vec<Interjection>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Interjection {
    pub voice: String,
    pub words: String,
}
