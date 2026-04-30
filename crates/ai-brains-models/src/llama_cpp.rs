use crate::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelError,
    ModelProvider, Result,
};
use async_trait::async_trait;
use serde::Serialize;

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct LlamaCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Serialize)]
struct LlamaEmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
}

pub struct LlamaCppProvider {
    endpoint: String,
    model: String,
}

impl LlamaCppProvider {
    pub fn new(endpoint: String, model: String) -> Self {
        Self { endpoint, model }
    }
}

#[async_trait]
impl ModelProvider for LlamaCppProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let client = reqwest::Client::new();
        // OpenAI compatible chat/completions
        let system_prompt = request
            .system_prompt
            .as_deref()
            .unwrap_or("You are a helpful assistant.");
        let body = LlamaCompletionRequest {
            model: &self.model,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user",
                    content: &request.prompt,
                },
            ],
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stream: false,
        };

        let res = client
            .post(format!("{}/v1/chat/completions", self.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::Network(e.to_string()))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(ModelError::Provider(format!(
                "llama.cpp (completions) returned {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| ModelError::Provider(e.to_string()))?;

        let text = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                ModelError::Provider("Missing choices[0].message.content field".to_string())
            })?
            .to_string();

        Ok(CompletionResponse {
            text,
            model: self.model.clone(),
        })
    }

    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        let client = reqwest::Client::new();
        // OpenAI compatible embeddings
        let body = LlamaEmbeddingRequest {
            model: &self.model,
            input: &request.text,
        };

        let res = client
            .post(format!("{}/v1/embeddings", self.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::Network(e.to_string()))?;

        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            return Err(ModelError::Provider(format!(
                "llama.cpp (embeddings) returned {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| ModelError::Provider(e.to_string()))?;

        let vector = json["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| ModelError::Provider("Missing data[0].embedding field".to_string()))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(EmbeddingResponse { vector })
    }

    fn name(&self) -> &str {
        "llama-cpp"
    }

    fn is_local(&self) -> bool {
        true
    }
}
