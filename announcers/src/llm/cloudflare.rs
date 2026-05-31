//! Cloudflare Workers AI-backed [`Commentator`] implementation.
//!
//! Uses the OpenAI-compatible chat API to run models like
//! `@cf/meta/llama-3.2-3b-instruct` on Cloudflare's infrastructure.
//! Cheapest option: free tier (10k neurons/day ~200-500 commentary
//! generations). Beyond that ~$0.01 per 1k neurons.
//!
//! # Environment variables
//!
//! - `CLOUDFLARE_API_TOKEN` — API token with Workers AI permission
//! - `CLOUDFLARE_ACCOUNT_ID` — your Cloudflare account ID

use async_trait::async_trait;
use futures::stream::Stream;
use std::pin::Pin;

use crate::llm::{Commentator, SYSTEM_PROMPT, parse_response};
use crate::types::{
    BroadcastPackage, CommentaryError, CommentaryLine, CommentarySegment,
};

/// Default model — 3B params, cheapest option on Cloudflare AI.
const DEFAULT_MODEL: &str = "@cf/meta/llama-3.2-3b-instruct";

/// Cloudflare AI API base URL.
const API_BASE: &str = "https://api.cloudflare.com/client/v4/accounts";

/// A Cloudflare Workers AI-backed commentator.
pub struct CloudflareCommentator {
    model: String,
    account_id: String,
    api_token: String,
    client: reqwest::Client,
}

impl CloudflareCommentator {
    /// Create a new Cloudflare commentator, reading credentials from
    /// `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` env vars.
    pub fn from_env() -> Result<Self, CommentaryError> {
        let api_token = std::env::var("CLOUDFLARE_API_TOKEN").map_err(|_| {
            CommentaryError::Generate(
                "CLOUDFLARE_API_TOKEN env var not set".into(),
            )
        })?;
        let account_id = std::env::var("CLOUDFLARE_ACCOUNT_ID").map_err(|_| {
            CommentaryError::Generate(
                "CLOUDFLARE_ACCOUNT_ID env var not set".into(),
            )
        })?;
        Ok(Self {
            model: DEFAULT_MODEL.into(),
            account_id,
            api_token,
            client: reqwest::Client::new(),
        })
    }

    /// Create a new Cloudflare commentator with explicit credentials.
    pub fn new(
        account_id: impl Into<String>,
        api_token: impl Into<String>,
    ) -> Self {
        Self {
            model: DEFAULT_MODEL.into(),
            account_id: account_id.into(),
            api_token: api_token.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Return the current model name.
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Set a different model (e.g. `@cf/meta/llama-3.1-8b-instruct`).
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// Build the chat messages from a broadcast package.
    fn build_messages(&self, package: &BroadcastPackage) -> serde_json::Value {
        let system = crate::llm::SYSTEM_PROMPT;
        let phase_data = ollama_format_prompt(package);

        serde_json::json!({
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": format!(
                    "Here is the current phase data:\n\n{phase_data}\n\n\
                     Generate the interleaved broadcast script now, \
                     using [VERITY], [REX], and [FLASH] tags."
                )}
            ]
        })
    }
}

#[async_trait]
impl Commentator for CloudflareCommentator {
    async fn generate(&self, package: &BroadcastPackage) -> Result<CommentarySegment, CommentaryError> {
        let body = self.build_messages(package);
        let url = format!(
            "{}/{}/ai/run/{}",
            API_BASE, self.account_id, self.model
        );

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .json(&body)
            .send()
            .await
            .map_err(|e| CommentaryError::Generate(format!("Cloudflare request failed: {e}")))?;

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| CommentaryError::Generate(format!("Cloudflare response parse failed: {e}")))?;

        // Cloudflare returns { success: bool, result: { response: str } }
        let text = data["result"]["response"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let lines = crate::llm::parse_response(&text);
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
        // Cloudflare doesn't support streaming for all models.
        // Fall back to batch.
        let segment = futures::executor::block_on(self.generate(package));
        let items: Vec<Result<CommentaryLine, CommentaryError>> = match segment {
            Ok(seg) => seg.lines.into_iter().map(Ok).collect(),
            Err(e) => vec![Err(e)],
        };
        Box::pin(futures::stream::iter(items))
    }
}

// Re-use the prompt formatting and parsing from the ollama module.
fn ollama_format_prompt(package: &BroadcastPackage) -> String {
    let mut body = String::new();

    body.push_str(&format!(
        "=== PHASE CONTEXT ===\n\
         Day {} — {} phase\n\
         {} tributes remaining\n\n",
        package.header.day,
        package.header.phase,
        package.header.alive_count,
    ));

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

    if !package.header.hot_zones.is_empty() {
        body.push_str("=== HOT ZONES ===\n");
        for zone in &package.header.hot_zones {
            body.push_str(&format!("• {} — {}\n", zone.name, zone.activity_level));
        }
        body.push('\n');
    }

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

    body.push_str("=== PHASE EVENTS ===\n");
    for event in &package.events {
        let icon = match event.kind {
            crate::types::EventKind::Death => "☠️",
            crate::types::EventKind::Combat => "⚔️",
            crate::types::EventKind::Allied => "🤝",
            crate::types::EventKind::Betrayal => "🗡️",
            crate::types::EventKind::Hazard => "🌪️",
            crate::types::EventKind::Item => "🎒",
            crate::types::EventKind::Movement => "🚶",
            crate::types::EventKind::Sponsor => "🎁",
            crate::types::EventKind::State => "📊",
            crate::types::EventKind::Other => "📌",
        };
        body.push_str(&format!("{} {}\n", icon, event.prose));
    }
    body.push('\n');

    if !package.histories.is_empty() {
        body.push_str("=== TRIBUTE HISTORIES ===\n");
        for t in &package.histories {
            let status_icon = if t.status == "alive" { "🟢" } else { "💀" };
            body.push_str(&format!(
                "{} {} (D{}) — {}, {}, at {}\n",
                status_icon, t.name, t.district, t.status, t.injury_level, t.location,
            ));
            for h in &t.highlights {
                body.push_str(&format!("  ★ {h}\n"));
            }
            for ev in t.notable_events.iter().take(5) {
                body.push_str(&format!("  • {ev}\n"));
            }
            if t.notable_events.len() > 5 {
                body.push_str(&format!("  ... ({} more events)\n", t.notable_events.len() - 5));
            }
            body.push('\n');
        }
    }

    body
}
