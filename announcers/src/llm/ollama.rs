//! Ollama-backed [`Commentator`] implementation.
//!
//! Serializes a [`BroadcastPackage`] into a structured prompt, sends it to
//! a local Ollama instance, and parses the response into `[VERITY]`/`[REX]`
//! tagged commentary lines.

use async_trait::async_trait;
use futures::stream::Stream;
use futures::StreamExt;
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use std::pin::Pin;

use crate::llm::Commentator;
use crate::types::{
    BroadcastPackage, CommentaryError, CommentaryLine, CommentarySegment, EventKind,
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
        let mut body = String::new();

        // ── Phase context ──
        body.push_str(&format!(
            "=== PHASE CONTEXT ===\n\
             {} tributes remaining\n\n",
            package.header.alive_count,
        ));

        // ── Hot streaks ──
        if !package.header.killing_sprees.is_empty() {
            body.push_str("=== 🔥 HOT STREAKS ===\n");
            for spree in &package.header.killing_sprees {
                body.push_str(&format!(
                    "🔥 {} (D{}) is {} — {} kills in a row!\n",
                    spree.name, spree.district, spree.label, spree.streak,
                ));
            }
            body.push('\n');
        }

        // ── Hot zones ──
        if !package.header.hot_zones.is_empty() {
            body.push_str("=== HOT ZONES ===\n");
            for zone in &package.header.hot_zones {
                body.push_str(&format!(
                    "• {} — {}\n",
                    zone.name, zone.activity_level,
                ));
            }
            body.push('\n');
        }

        // ── Kill leaders (this phase) ──
        if !package.header.kill_leaders.is_empty() {
            body.push_str("=== KILL LEADERS ===\n");
            for leader in &package.header.kill_leaders {
                body.push_str(&format!(
                    "• {} (D{}) — {} kill{}\n",
                    leader.name,
                    leader.district,
                    leader.kill_count,
                    if leader.kill_count == 1 { "" } else { "s" },
                ));
            }
            body.push('\n');
        }

        // ── Phase events ──
        body.push_str("=== PHASE EVENTS ===\n");
        for event in &package.events {
            let icon = event_icon(event.kind);
            body.push_str(&format!("{} {}\n", icon, event.prose));
            if let Some(ref structured) = event.structured {
                // Only include structured data for complex events (death, combat).
                if matches!(event.kind, EventKind::Death | EventKind::Combat | EventKind::Betrayal) {
                    if let Some(s) = structured.as_str() {
                        body.push_str(&format!("  ({s})\n"));
                    }
                }
            }
        }
        body.push('\n');

        // ── Tribute histories ──
        if !package.histories.is_empty() {
            body.push_str("=== TRIBUTE HISTORIES ===\n");
            for t in &package.histories {
                let status_icon = if t.status == "alive" { "🟢" } else { "💀" };
                body.push_str(&format!(
                    "{} {} (D{}) — {}, {}, at {}\n",
                    status_icon, t.name, t.district, t.status, t.injury_level, t.location,
                ));
                // Highlights (permanent).
                for h in &t.highlights {
                    body.push_str(&format!("  ★ {h}\n"));
                }
                // Recent notable events (first 5, newest first).
                let recent: Vec<&String> = t.notable_events.iter().take(5).collect();
                if !recent.is_empty() {
                    for (i, ev) in recent.iter().enumerate() {
                        body.push_str(&format!("  {}. {ev}\n", i + 1));
                    }
                }
                if t.notable_events.len() > 5 {
                    body.push_str(&format!(
                        "  ... ({} more events)\n",
                        t.notable_events.len() - 5
                    ));
                }
                body.push('\n');
            }
        }

        format!(
            r#"{SYSTEM_PROMPT}

Here is the current phase data:

{body}Generate the interleaved broadcast script now, using [VERITY] and [REX] tags.
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

    fn generate_stream(
        &self,
        package: &BroadcastPackage,
    ) -> Pin<Box<dyn Stream<Item = Result<CommentaryLine, CommentaryError>> + Send>> {
        let prompt = self.build_prompt(package);
        let model = self.model.clone();
        let client = self.client.clone();

        // Use unfold to turn the Ollama token stream into a CommentaryLine stream.
        let stream = futures::stream::unfold(
            OllamaStreamState {
                client,
                model,
                prompt,
                buffer: String::new(),
                ollama_stream: None,
                done: false,
            },
            |mut state| async move {
                if state.done {
                    let line = flush_buffer(&mut state.buffer);
                    return line.map(|l| (Ok(l), state));
                }

                // Lazily start the Ollama stream on first poll.
                if state.ollama_stream.is_none() {
                    let request = GenerationRequest::new(state.model.clone(), state.prompt.clone());
                    match state.client.generate_stream(request).await {
                        Ok(s) => {
                            // Convert the complex ollama stream into a simple
                            // string stream by extracting `.response` from each
                            // `GenerationResponse`.
                            let string_stream: Pin<
                                Box<dyn Stream<Item = Result<String, ollama_rs::error::OllamaError>> + Send>,
                            > = Box::pin(s.flat_map(move |result| {
                                let items: Vec<Result<String, ollama_rs::error::OllamaError>> = match result {
                                    Ok(chunk) => chunk.into_iter().map(|resp| Ok(resp.response)).collect(),
                                    Err(e) => vec![Err(e)],
                                };
                                futures::stream::iter(items)
                            }));
                            state.ollama_stream = Some(string_stream);
                        }
                        Err(e) => {
                            state.done = true;
                            return Some((
                                Err(CommentaryError::Generate(format!(
                                    "Ollama stream failed: {e}"
                                ))),
                                state,
                            ));
                        }
                    }
                }

                // Pull tokens from Ollama, buffering by line.
                if let Some(ref mut stream) = state.ollama_stream {
                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(chunk) => {
                                state.buffer.push_str(&chunk);
                                // Yield complete lines ending in \n.
                                while let Some(pos) = state.buffer.find('\n') {
                                    let line = state.buffer[..pos].trim().to_string();
                                    state.buffer = state.buffer[pos + 1..].to_string();
                                    if let Some(parsed) = parse_line(&line) {
                                        return Some((Ok(parsed), state));
                                    }
                                }
                            }
                            Err(e) => {
                                state.done = true;
                                return Some((
                                    Err(CommentaryError::Generate(format!(
                                        "Ollama stream token error: {e}"
                                    ))),
                                    state,
                                ));
                            }
                        }
                    }
                }

                // Ollama stream ended naturally.
                state.done = true;
                let line = flush_buffer(&mut state.buffer);
                if let Some(l) = line {
                    return Some((Ok(l), state));
                }
                None
            },
        );

        Box::pin(stream)
    }
}

/// Internal state for the `generate_stream` unfold.
struct OllamaStreamState {
    client: Ollama,
    model: String,
    prompt: String,
    buffer: String,
    /// Boxed to avoid naming the complex stream type from ollama-rs.
    ollama_stream:
        Option<Pin<Box<dyn Stream<Item = Result<String, ollama_rs::error::OllamaError>> + Send>>>,
    done: bool,
}

/// Try to extract a `[VERITY]` or `[REX]` line from the buffer.
fn flush_buffer(buffer: &mut String) -> Option<CommentaryLine> {
    if buffer.trim().is_empty() {
        return None;
    }
    let line = std::mem::take(buffer);
    parse_line(line.trim())
}

/// Parse a single line as a `[VERITY]` or `[REX]` utterance.
fn parse_line(line: &str) -> Option<CommentaryLine> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    line.strip_prefix("[VERITY]")
        .or_else(|| line.strip_prefix("[REX]"))
        .map(|text| CommentaryLine {
            speaker: if line.starts_with("[VERITY]") {
                "Verity".into()
            } else {
                "Rex".into()
            },
            text: text.trim().to_string(),
        })
}

/// Returns a single-character icon for an event kind, giving the LLM a
/// visual cue about the event type in the formatted prompt.
fn event_icon(kind: EventKind) -> &'static str {
    match kind {
        EventKind::Death => "☠️",
        EventKind::Combat => "⚔️",
        EventKind::Allied => "🤝",
        EventKind::Betrayal => "🗡️",
        EventKind::Hazard => "🌪️",
        EventKind::Item => "🎒",
        EventKind::Movement => "🚶",
        EventKind::Sponsor => "🎁",
        EventKind::State => "📊",
        EventKind::Other => "📌",
    }
}
