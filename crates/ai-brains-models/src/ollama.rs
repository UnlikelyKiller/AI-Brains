use crate::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelError,
    ModelProvider, Result,
};
use async_trait::async_trait;
use serde::Serialize;

#[derive(Serialize)]
struct OllamaOptions {
    num_predict: Option<u32>,
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct OllamaCompletionRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    system: &'a Option<String>,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaEmbeddingRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

pub struct OllamaProvider {
    endpoint: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(endpoint: String, model: String) -> Self {
        Self { endpoint, model }
    }
}

#[async_trait]
impl ModelProvider for OllamaProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse> {
        let client = reqwest::Client::new();
        let body = OllamaCompletionRequest {
            model: &self.model,
            prompt: &request.prompt,
            system: &request.system_prompt,
            stream: false,
            options: OllamaOptions {
                num_predict: request.max_tokens,
                temperature: request.temperature,
            },
        };

        let res = client
            .post(format!("{}/api/generate", self.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(ModelError::Provider(format!(
                "Ollama returned {}",
                res.status()
            )));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| ModelError::Provider(e.to_string()))?;
        let text = json["response"]
            .as_str()
            .ok_or_else(|| ModelError::Provider("Missing response field".to_string()))?
            .to_string();

        Ok(CompletionResponse {
            text,
            model: self.model.clone(),
        })
    }

    async fn embed(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        let client = reqwest::Client::new();
        let body = OllamaEmbeddingRequest {
            model: &self.model,
            prompt: &request.text,
        };

        let res = client
            .post(format!("{}/api/embeddings", self.endpoint))
            .json(&body)
            .send()
            .await
            .map_err(|e| ModelError::Network(e.to_string()))?;

        if !res.status().is_success() {
            return Err(ModelError::Provider(format!(
                "Ollama returned {}",
                res.status()
            )));
        }

        let json: serde_json::Value = res
            .json()
            .await
            .map_err(|e| ModelError::Provider(e.to_string()))?;
        let vector = json["embedding"]
            .as_array()
            .ok_or_else(|| ModelError::Provider("Missing embedding field".to_string()))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(EmbeddingResponse { vector })
    }

    fn name(&self) -> &str {
        "ollama"
    }

    fn is_local(&self) -> bool {
        true
    }
}
