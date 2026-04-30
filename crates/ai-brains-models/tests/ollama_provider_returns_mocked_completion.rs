#![allow(clippy::disallowed_methods)]

use ai_brains_models::ollama::OllamaProvider;
use ai_brains_models::{CompletionRequest, ModelProvider};
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_ollama_completion() -> Result<(), Box<dyn std::error::Error>> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "response": "Hello from Ollama!"
        })))
        .mount(&mock_server)
        .await;

    let provider = OllamaProvider::new(mock_server.uri(), "llama3".to_string());
    let request = CompletionRequest {
        prompt: "Hi".to_string(),
        system_prompt: None,
        max_tokens: None,
        temperature: None,
    };

    let response = provider.complete(request).await?;
    assert_eq!(response.text, "Hello from Ollama!");
    assert_eq!(response.model, "llama3");

    Ok(())
}
