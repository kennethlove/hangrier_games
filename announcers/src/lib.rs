use ollama_rs::generation::completion::request::GenerationRequest;
use ollama_rs::Ollama;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnnouncerError {
    #[error("failed to generate a response")]
    FailedToGenerateResponse,
}

pub static MODEL: &str = "announcers";

pub static ANNOUNCER_PROMPT: &str = r#"
You are writing a sports broadcast team covering the newest Hunger Games.
Provide the spoken script for Verity and Rex directly with no summaries or conclusions.

Now, here is this cycle's log entry:
"#;

pub fn prompt(log: &str) -> String {
    format!(r#"
{ANNOUNCER_PROMPT}

{log}

Generate just the spoken script for Verity and Rex.
No notes, description, summary, or commentary is needed."#)
}

pub async fn summarize(log: &str) -> Result<String, AnnouncerError> {
    let log_prompt = prompt(log);

    let ollama = Ollama::default();
    let res = ollama.generate(
        GenerationRequest::new(MODEL.into(), log_prompt)
    ).await;

    if let Ok(res) = res {
        Ok(res.response.to_string())
    } else {
        Err(AnnouncerError::FailedToGenerateResponse)
    }
}
