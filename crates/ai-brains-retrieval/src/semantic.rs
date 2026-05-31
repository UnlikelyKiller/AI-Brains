use crate::errors::{Result, RetrievalError};
use crate::privacy_filter::is_injectable_privacy;
use crate::recall::RecallHit;
use ai_brains_core::privacy::Privacy;
use ai_brains_models::ModelProvider;
use ai_brains_store::VaultConnection;
use rusqlite::params_from_iter;

type EmbeddedMemory = (String, String, Privacy, Vec<u8>);

/// Perform semantic search over pinned memories with non-null embeddings.
/// Fetches an embedding for the query via LlamaCppProvider, then computes
/// cosine similarity against stored embedding BLOBs.
pub fn semantic_search(
    conn: &VaultConnection,
    query: &str,
    limit: usize,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
) -> Result<Vec<RecallHit>> {
    let query_embedding = fetch_embedding(query)?;
    let memories = fetch_pinned_embeddings(conn, project_id, session_id)?;

    let mut scored: Vec<(f64, RecallHit)> = memories
        .into_iter()
        .filter_map(|(memory_id, content, privacy, embedding_bytes)| {
            let emb = bytes_to_f32_vec(&embedding_bytes)?;
            let sim = cosine_similarity(&query_embedding, &emb)?;
            Some((
                sim,
                RecallHit {
                    memory_id,
                    content,
                    source: "semantic".to_string(),
                    score: Some(sim),
                    privacy: Some(privacy),
                },
            ))
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);

    Ok(scored.into_iter().map(|(_, hit)| hit).collect())
}

fn fetch_embedding(text: &str) -> Result<Vec<f32>> {
    let text = text.to_string();
    let handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| RetrievalError::Model(format!("runtime creation failed: {}", e)))?;
        let endpoint = std::env::var("AI_BRAINS_EMBEDDING_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8083".to_string());
        let provider = ai_brains_models::llama_cpp::LlamaCppProvider::new(
            endpoint,
            "nomic-embed-text-v1.5".to_string(),
        );
        let req = ai_brains_models::EmbeddingRequest { text };
        rt.block_on(provider.embed(req))
            .map(|res| res.vector)
            .map_err(|e| RetrievalError::Model(format!("embedding request failed: {}", e)))
    });

    handle
        .join()
        .map_err(|e| RetrievalError::Model(format!("embedding thread panicked: {:?}", e)))?
}

fn fetch_pinned_embeddings(
    conn: &VaultConnection,
    project_id: Option<ai_brains_core::ids::ProjectId>,
    session_id: Option<ai_brains_core::ids::SessionId>,
) -> Result<Vec<EmbeddedMemory>> {
    let conn = conn.lock()?;

    let mut sql = "SELECT mp.memory_id, mp.content, mp.privacy, mp.embedding
        FROM memory_projection mp
        LEFT JOIN session_projection sp ON mp.session_id = sp.session_id
        WHERE mp.status = 'pinned' AND mp.embedding IS NOT NULL"
        .to_string();

    let mut params: Vec<rusqlite::types::Value> = Vec::new();

    if let Some(sid) = session_id {
        sql.push_str(" AND mp.session_id = ?");
        params.push(sid.to_string().into());
    }

    if let Some(pid) = project_id {
        sql.push_str(" AND (sp.project_id = ? OR mp.project_id = ?)");
        let pid_str = pid.to_string();
        params.push(pid_str.clone().into());
        params.push(pid_str.into());
    }

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(params))?;
    let mut results = Vec::new();

    while let Some(row) = rows.next()? {
        let memory_id: String = row.get(0)?;
        let content: String = row.get(1)?;
        let privacy_str: String = row.get(2)?;
        let embedding: Vec<u8> = row.get(3)?;

        if !is_injectable_privacy(&privacy_str) {
            continue;
        }

        let privacy: Privacy = serde_json::from_str(&privacy_str).unwrap_or(Privacy::LocalOnly);
        results.push((memory_id, content, privacy, embedding));
    }

    Ok(results)
}

fn bytes_to_f32_vec(bytes: &[u8]) -> Option<Vec<f32>> {
    if !bytes.len().is_multiple_of(4) {
        return None;
    }
    let mut vec = Vec::with_capacity(bytes.len() / 4);
    for chunk in bytes.chunks_exact(4) {
        let arr: [u8; 4] = chunk.try_into().ok()?;
        vec.push(f32::from_le_bytes(arr));
    }
    Some(vec)
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> Option<f64> {
    if a.len() != b.len() || a.is_empty() {
        return None;
    }
    let mut dot = 0.0f64;
    let mut norm_a = 0.0f64;
    let mut norm_b = 0.0f64;
    for i in 0..a.len() {
        let av = a[i] as f64;
        let bv = b[i] as f64;
        dot += av * bv;
        norm_a += av * av;
        norm_b += bv * bv;
    }
    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        return None;
    }
    Some(dot / denom)
}
