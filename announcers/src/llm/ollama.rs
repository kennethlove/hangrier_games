//! Ollama-backed [`Commentator`] implementation.
//!
//! Serializes a [`BroadcastPackage`] into a structured prompt, sends it to
//! a local Ollama instance, and parses the response into `[VERITY]`/`[REX]`
//! tagged commentary lines.

use async_trait::async_trait;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;

use crate::llm::Commentator;
use crate::types::{
    BroadcastPackage, CommentaryError, CommentaryLine, CommentarySegment,
};

/// Default Ollama model name to use for commentary generation.
const DEFAULT_MODEL: &str = "announcers";

/// System prompt establishing the commentator voices.
const SYSTEM_PROMPT: &str = r#"You are a live sports broadcast team covering the Hunger Games.
Provide the spoken script for Verity (play-by-play) and Rex (color commentary).

Format each line with [VERITY] or [REX] at the start, like:

[VERITY] And here we are in the arena, folks — what a bloodbath it's been so far!
[REX] I haven't seen carnage like this since the 47th Games, Verity. Absolutely brutal.
[VERITY] Katniss from District 12 is really making a statement with that bow.

Generate 4-8 lines of interleaved back-and-forth dialogue covering the highlights.
No narration, no descriptions, no stage directions — just the spoken script.
"#;

/// An Ollama-backed commentator.
pub struct OllamaCommentator {
    model: String,
    client: Ollama,
}

impl OllamaCommentator {
    /// Create a new Ollama commentator with the default model and localhost
    /// Ollama instance.
    pub fn new() -> Self {
        Self {
            model: DEFAULT_MODEL.into(),
            client: Ollama::default(),
        }
    }

    /// Create a new Ollama commentator with a custom model name.
    pub fn with_model(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            client: Ollama::default(),
        }
    }

    /// Create a new Ollama commentator with a custom model and client.
    pub fn new_with_client(model: impl Into<String>, client: Ollama) -> Self {
        Self {
            model: model.into(),
            client,
        }
    }

    /// Build the full prompt from a broadcast package.
    fn build_prompt(&self, package: &BroadcastPackage) -> String {
        let serialized = serde_json::to_string_pretty(package)
            .unwrap_or_else(|_| "<serialization error>".into());

        format!(
            r#"{SYSTEM_PROMPT}

Here is the current phase data:

{serialized}

Generate the interleaved broadcast script now, using [VERITY] and [REX] tags.
"#,
        )
    }

    /// Parse the raw LLM response into commentary lines.
    fn parse_response(text: &str) -> Vec<CommentaryLine> {
        text.lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }

                line.strip_prefix("[VERITY]")
                    .map(|text| CommentaryLine {
                        speaker: "Verity".into(),
                        text: text.trim().to_string(),
                    })
                    .or_else(|| {
                        line.strip_prefix("[REX]").map(|text| CommentaryLine {
                            speaker: "Rex".into(),
                            text: text.trim().to_string(),
                        })
                    })
            })
            .collect()
    }
}

impl Default for OllamaCommentator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Commentator for OllamaCommentator {
    async fn generate(&self, package: &BroadcastPackage) -> Result<CommentarySegment, CommentaryError> {
        let prompt = self.build_prompt(package);
        let request = GenerationRequest::new(self.model.clone(), prompt);

        let response = self
            .client
            .generate(request)
            .await
            .map_err(|e| CommentaryError::Generate(format!("Ollama generation failed: {e}")))?;

        let lines = Self::parse_response(&response.response);
        let generated_at = chrono::Utc::now();

        Ok(CommentarySegment {
            id: uuid::Uuid::new_v4().to_string(),
            game_id: String::new(),                       // filled by caller
            day: 0,                                        // filled by caller
            phase: String::new(),                          // filled by caller
            lines,
            generated_at,
            model_used: self.model.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, EventLine, GameStateSnapshot};

    fn sample_package() -> BroadcastPackage {
        BroadcastPackage {
            header: GameStateSnapshot {
                alive_count: 12,
                kill_leaders: vec![],
                alliances: vec![],
                hot_zones: vec![],
            killing_sprees: vec![],
            },
            events: vec![
                EventLine {
                    kind: EventKind::Death,
                    prose: "Cato killed Peeta.".into(),
                    structured: None,
                },
                EventLine {
                    kind: EventKind::Combat,
                    prose: "Katniss wounded Marvel.".into(),
                    structured: None,
                },
            ],
            histories: vec![],
        }
    }

    #[test]
    fn prompt_contains_system_and_package() {
        let commentator = OllamaCommentator::new();
        let pkg = sample_package();
        let prompt = commentator.build_prompt(&pkg);

        assert!(prompt.contains("live sports broadcast team"));
        assert!(prompt.contains("Cato killed Peeta."));
        assert!(prompt.contains("alive_count"));
    }

    #[test]
    fn parse_verity_rex_lines() {
        let input = "\
[VERITY] And here we go!
[REX] This is intense.
[VERITY] Katniss takes aim.

Some random text that should be ignored.

[REX] What a shot!";
        let lines = OllamaCommentator::parse_response(input);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].speaker, "Verity");
        assert_eq!(lines[0].text, "And here we go!");
        assert_eq!(lines[1].speaker, "Rex");
        assert_eq!(lines[1].text, "This is intense.");
        assert_eq!(lines[2].speaker, "Verity");
        assert_eq!(lines[3].speaker, "Rex");
        assert_eq!(lines[3].text, "What a shot!");
    }

    #[test]
    fn parse_empty_response() {
        assert_eq!(OllamaCommentator::parse_response("").len(), 0);
    }

    #[test]
    fn parse_skips_non_tagged_lines() {
        let input = "\
[VERITY] Hello.
Narrator: something happened.
[REX] Goodbye.";
        let lines = OllamaCommentator::parse_response(input);
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn build_prompt_handles_empty_package() {
        let commentator = OllamaCommentator::new();
        let pkg = BroadcastPackage {
            header: GameStateSnapshot {
                alive_count: 24,
                kill_leaders: vec![],
                alliances: vec![],
                hot_zones: vec![],
            killing_sprees: vec![],
            },
            events: vec![],
            histories: vec![],
        };
        let prompt = commentator.build_prompt(&pkg);
        assert!(prompt.contains("alive_count"));
        assert!(prompt.contains("24"));
    }
}
