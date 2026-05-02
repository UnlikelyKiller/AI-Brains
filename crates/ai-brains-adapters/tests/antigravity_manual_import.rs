use ai_brains_adapters::{
    adapter_capability, apply_idempotent_patch, install_scope, manual_import_instructions,
    wrapper_command, AdapterKind, CapabilityLevel,
};

#[test]
fn antigravity_manual_import() {
    let capability = adapter_capability(AdapterKind::Antigravity);
    let instructions = manual_import_instructions();
    let patched = apply_idempotent_patch("alpha", "beta");

    assert_eq!(capability.level, CapabilityLevel::Partial);
    assert!(instructions.contains("nightly"));
    assert!(instructions.contains("antigravity-import"));
    assert_eq!(install_scope(), "user");
    assert_eq!(wrapper_command("ai-brains-cli"), "ai-brains-cli ingest");
    assert_eq!(patched, "alpha\nbeta");
}
