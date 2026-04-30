use crate::{
    CompletionRequest, CompletionResponse, EmbeddingRequest, EmbeddingResponse, ModelError,
    ModelProvider, Result,
};
use async_trait::async_trait;
use std::sync::Mutex;

pub struct MockProvider {
    pub responses: Mutex<Vec<CompletionResponse>>,
    pub is_local: bool,
}

impl MockProvider {
    pub fn new(responses: Vec<CompletionResponse>) -> Self {
        Self {
            responses: Mutex::new(responses),
            is_local: true,
        }
    }
}

#[async_trait]
impl ModelProvider for MockProvider {
    async fn complete(&self, _request: CompletionRequest) -> Result<CompletionResponse> {
        let mut responses = self
            .responses
            .lock()
            .map_err(|e| ModelError::Provider(format!("mock response lock poisoned: {e}")))?;
        if responses.is_empty() {
            return Ok(CompletionResponse {
                text: "No more mock responses".to_string(),
                model: "mock".to_string(),
            });
        }
        Ok(responses.remove(0))
    }

    async fn embed(&self, _request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        Ok(EmbeddingResponse {
            vector: vec![0.0; 1536],
        })
    }

    fn name(&self) -> &str {
        "mock"
    }

    fn is_local(&self) -> bool {
        self.is_local
    }
}
