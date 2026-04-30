#![allow(clippy::disallowed_methods)]

use ai_brains_core::privacy::Privacy;
use ai_brains_models::mock::MockProvider;
use ai_brains_models::registry::ProviderRegistry;
use ai_brains_models::CompletionResponse;
use std::sync::Mutex;

#[test]
fn test_registry_blocks_cloud_for_local_only() {
    let mut registry = ProviderRegistry::new();

    // Register a cloud-only provider
    registry.register(Box::new(MockProvider {
        responses: Mutex::new(vec![CompletionResponse {
            text: "cloud response".to_string(),
            model: "mock".to_string(),
        }]),
        is_local: false,
    }));

    // Try to select for LocalOnly
    let result = registry.select_provider(&Privacy::LocalOnly);
    assert!(result.is_err());
}

#[test]
fn test_registry_allows_local_for_local_only() {
    let mut registry = ProviderRegistry::new();

    // Register a local provider
    registry.register(Box::new(MockProvider {
        responses: Mutex::new(vec![CompletionResponse {
            text: "local response".to_string(),
            model: "mock".to_string(),
        }]),
        is_local: true,
    }));

    // Select for LocalOnly
    let result = registry.select_provider(&Privacy::LocalOnly);
    assert!(result.is_ok());
    assert!(result.unwrap().is_local());
}
