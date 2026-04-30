use ai_brains_graph::LadybugVault;
use tempfile::tempdir;

#[test]
fn test_schema_initializes() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempdir()?;
    let db_path = dir.path().join("test.db");

    // First open initializes schema
    {
        let _vault = LadybugVault::open(&db_path)?;
    }

    // Second open should not error on existing schema
    {
        let _vault = LadybugVault::open(&db_path)?;
    }

    Ok(())
}
