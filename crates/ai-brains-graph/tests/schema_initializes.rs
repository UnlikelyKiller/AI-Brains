use ai_brains_crypto::{DataKey, SqlCipherKey};
use ai_brains_graph::GraphVault;
use ai_brains_store::VaultConnection;
use tempfile::tempdir;

#[test]
fn test_schema_initializes() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.db");
    let key = SqlCipherKey::from_data_key(&DataKey::generate());

    // First open initializes schema via migrations
    {
        let conn = VaultConnection::open(&db_path, &key)?;
        conn.migrate()?;
        let _vault = GraphVault::new(conn);
    }

    // Second open should not error on existing schema
    {
        let conn = VaultConnection::open(&db_path, &key)?;
        conn.migrate()?;
        let _vault = GraphVault::new(conn);
    }

    Ok(())
}
