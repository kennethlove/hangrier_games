use futures::StreamExt;
use futures::stream::{BoxStream, Stream};
use ollama_rs::Ollama;
use ollama_rs::generation::completion::request::GenerationRequest;
use std::convert::Infallible;
use std::pin::Pin;
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
    format!(
        r#"
{ANNOUNCER_PROMPT}

{log}

Generate just the spoken script for Verity and Rex.
No notes, description, summary, or commentary is needed."#
    )
}

pub async fn summarize(log: &str) -> Result<String, AnnouncerError> {
    let log_prompt = prompt(log);

    let ollama = Ollama::default();
    let res = ollama
        .generate(GenerationRequest::new(MODEL.into(), log_prompt))
        .await;

    if let Ok(res) = res {
        Ok(res.response.to_string())
    } else {
        Err(AnnouncerError::FailedToGenerateResponse)
    }
}

pub async fn summarize_stream(log: &str) -> Pin<Box<dyn Stream<Item = Result<String, String>> + Send>> {
    let log_prompt = prompt(log);
    let ollama = Ollama::default();

    let stream = async_stream::stream! {
        let gen_request = GenerationRequest::new(MODEL.into(), log_prompt);

        let mut stream = match ollama.generate_stream(gen_request).await {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Error creating stream: {}", e);
                yield Err(format!("Error creating stream: {}", e));
                return;
            }
        };

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    for resp in response {
                        yield Ok(resp.response);
                    }
                }
                Err(e) => {
                    eprintln!("Error generating response: {}", e);
                    yield Err(format!("Error generating response: {}", e));
                    break;
                }
            }
        }
    };

    stream.boxed()
}
