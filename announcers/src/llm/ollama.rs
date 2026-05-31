//! Ollama-backed [`Commentator`] implementation.
//!
//! Serializes a [`BroadcastPackage`] into a structured prompt, sends it to
//! a local Ollama instance, and parses the response into `[VERITY]`/`[REX]`
//! tagged commentary lines.

use async_trait::async_trait;
use futures::stream::Stream;
use futures::StreamExt;
use std::pin::Pin;

use crate::llm::Commentator;
use crate::types::{
    BroadcastPackage, CommentaryError, CommentaryLine, CommentarySegment, EventKind,
};

/// Default Ollama model name — a custom `announcers` Modelfile built
/// from phi3:3.8b. Created via: ollama create announcers -f Modelfile
const DEFAULT_MODEL: &str = "announcers";

/// System prompt establishing the commentator voices.
const SYSTEM_PROMPT: &str = r#"You are the Capitol's Hunger Games broadcast team:

VERITY — play-by-play. Sharp, dramatic, paints the picture. Calls the action.
REX — color commentator. Cynical, theatrical, darkly amused. Reacts to the drama.
FLASH — technical analyst. Analytical, precise. Comments on combat technique, weaponry, gear, injuries, and arena tactics. Notices the details.

Tone: Ancient Rome colosseum meets pro wrestling. Theatrical, bloodthirsty, gripping. Refer to tributes by name and district. Build tension.

RULES:
- 4-6 exchanges (8-12 lines). Rotate through all three commentators.
- ONLY spoken dialogue — NO stage directions, actions, or descriptions in asterisks or parentheses.
- Do NOT write *screams*, *laughs*, *voice trembling*, etc. Just the words they say.
- ONLY use English. Do NOT output Chinese characters or any non-English text.

- End with a hook: what happens next, who to watch.
- Only reference events in the data. Do NOT invent kills. Dead tributes are dead.
- Vary your vocabulary. Do NOT repeat the same phrases across different exchanges.

Examples of the right TONE (not scripts to copy):

[VERITY] Cato from District 2 has his FOURTH kill! The Cornucopia is a slaughterhouse!
[REX] Cato is possessed! I haven't seen a feeding frenzy this brutal since the 63rd Games!
[FLASH] He's switching his grip mid-swing — that's not brute force, that's District 2 combat training. Textbook.

Now cover this phase.
"#;

/// An Ollama-backed commentator. Communicates with Ollama's REST API directly
/// via reqwest so we can pass `think: false` and other options not exposed by
/// the `ollama-rs` crate.
pub struct OllamaCommentator {
    model: String,
    base_url: String,
    client: reqwest::Client,
}

impl OllamaCommentator {
    /// Create a new Ollama commentator with the default model and localhost.
    pub fn new() -> Self {
        Self {
            model: DEFAULT_MODEL.into(),
            base_url: "http://localhost:11434".into(),
            client: reqwest::Client::new(),
        }
    }

    /// Create a new Ollama commentator with a custom model name.
    pub fn with_model(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            base_url: "http://localhost:11434".into(),
            client: reqwest::Client::new(),
        }
    }

    /// Build the full prompt from a broadcast package.
    pub fn build_prompt(&self, package: &BroadcastPackage) -> String {
        let mut body = String::new();

        // ── Phase context ──
        body.push_str(&format!(
            "=== PHASE CONTEXT ===\n\
             Day {} — {} phase\n\
             {} tributes remaining\n\n",
            package.header.day,
            package.header.phase,
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

{body}Generate the interleaved broadcast script now, using [VERITY], [REX], and [FLASH] tags.
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
        let body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "repeat_penalty": 1.5,
                "temperature": 0.8,
            },
        });

        let resp = self
            .client
            .post(format!("{}/api/generate", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| CommentaryError::Generate(format!("Ollama request failed: {e}")))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| CommentaryError::Generate(format!("Ollama response parse failed: {e}")))?;

        let text = data["response"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let lines = Self::parse_response(&text);
        let generated_at = chrono::Utc::now();

        Ok(CommentarySegment {
            id: uuid::Uuid::new_v4().to_string(),
            game_id: String::new(),
            day: 0,
            phase: String::new(),
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
        let base_url = self.base_url.clone();
        let model = self.model.clone();

        // Use unfold to turn the Ollama token stream into a CommentaryLine stream.
        let stream = futures::stream::unfold(
            OllamaStreamState {
                base_url,
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
                    // Start a streaming request via the Ollama REST API.
                    let body = serde_json::json!({
                        "model": state.model,
                        "prompt": state.prompt,
                        "stream": true,
                        "options": {
                            "repeat_penalty": 1.5,
                            "temperature": 0.8,
                        },
                    });
                    let client = reqwest::Client::new();
                    match client
                        .post(format!("{}/api/generate", state.base_url))
                        .json(&body)
                        .send()
                        .await
                    {
                        Ok(response) => {
                            // Convert the SSE stream into a string stream by
                            // reading each JSON line and extracting .response.
                            let byte_stream = response.bytes_stream();
                            let string_stream: Pin<
                                Box<dyn Stream<Item = Result<String, Box<dyn std::error::Error + Send>>> + Send>,
                            > = Box::pin(
                                byte_stream.map(|chunk_result| match chunk_result {
                                    Ok(bytes) => Ok(String::from_utf8_lossy(&bytes).to_string()),
                                    Err(e) => Err(Box::new(e) as Box<dyn std::error::Error + Send>),
                                }),
                            );

                            // Parse SSE JSON lines, extract .response fields.
                            let parsed = string_stream.flat_map(move |chunk_result| {
                                let items: Vec<Result<String, Box<dyn std::error::Error + Send>>> =
                                    match chunk_result {
                                        Ok(text) => text
                                            .lines()
                                            .filter_map(|line| {
                                                let line = line.trim();
                                                if line.is_empty() {
                                                    return None;
                                                }
                                                let val: serde_json::Value =
                                                    serde_json::from_str(line).ok()?;
                                                let token = val["response"].as_str()?.to_string();
                                                Some(Ok(token))
                                            })
                                            .collect(),
                                        Err(e) => vec![Err(e)],
                                    };
                                futures::stream::iter(items)
                            });
                            state.ollama_stream = Some(Box::pin(parsed));
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
    base_url: String,
    model: String,
    prompt: String,
    buffer: String,
    /// SSE-parsed token stream from the Ollama REST API.
    ollama_stream:
        Option<Pin<Box<dyn Stream<Item = Result<String, Box<dyn std::error::Error + Send>>> + Send>>>,
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
