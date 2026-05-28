use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OllamaClient {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaClient {
    pub fn new(
        base_url: impl Into<String>,
        model: impl Into<String>,
        timeout_seconds: u64,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .expect("failed to build reqwest client");

        Self {
            client,
            base_url: base_url.into(),
            model: model.into(),
        }
    }

    pub fn generate(&self, prompt: &str) -> Result<String> {
        let request = OllamaGenerateRequest {
            model: self.model.clone(),
            prompt: prompt.to_string(),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/generate", self.base_url))
            .json(&request)
            .send()
            .context("failed to send request to ollama")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .unwrap_or_else(|_| "<failed to read body>".to_string());

            anyhow::bail!(
                "ollama request failed with status {}: {}",
                status,
                body
            );
        }

        let response: OllamaGenerateResponse = response
            .json()
            .context("failed to parse ollama response json")?;

        Ok(response.response)
    }
}

#[derive(Debug, Clone, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_serialization_contains_expected_fields() {
        let request = OllamaGenerateRequest {
            model: "llama3.1".to_string(),
            prompt: "hello".to_string(),
            stream: false,
        };

        let json =
            serde_json::to_value(&request).unwrap();

        assert_eq!(json["model"], "llama3.1");
        assert_eq!(json["prompt"], "hello");
        assert_eq!(json["stream"], false);
    }

    #[test]
    fn response_deserialization_works() {
        let json = r#"
{
    "response": "grouped successfully"
}
"#;

        let response: OllamaGenerateResponse =
            serde_json::from_str(json).unwrap();

        assert_eq!(
            response.response,
            "grouped successfully"
        );
    }

    #[test]
    fn ollama_client_stores_configuration() {
        let client = OllamaClient::new(
            "http://127.0.0.1:11434/api",
            "llama3.1",
            300,
        );

        assert_eq!(
            client.base_url,
            "http://127.0.0.1:11434/api"
        );

        assert_eq!(client.model, "llama3.1");
    }
}
