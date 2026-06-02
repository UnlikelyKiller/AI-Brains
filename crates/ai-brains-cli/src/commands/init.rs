use crate::context::AppContext;
use ai_brains_store::QueryStore;

/// Initialize or re-validate the vault at `ctx.vault_path`.
///
/// Behavior (T73):
/// - The vault file is opened (and created if missing) by `AppContext::from_cli`
///   before this function runs, so by the time we get here, the file exists.
/// - "Empty vault" signal: `list_projects()` returns no rows. In that case
///   print `"Vault initialized successfully at <path>"` and exit 0.
/// - "Populated vault" signal: `list_projects()` returns 1+ rows. Refuse unless
///   `--force` is set. Print a clear refusal on stderr and return an error so
///   the CLI emits a structured `ApiResult::error` JSON envelope with exit 1.
/// - With `--force`, the above refusal is bypassed: print the success message
///   and exit 0 (the caller is asserting they understand the implications).
///
/// Note: we deliberately do not use file existence as the "is this vault
/// initialized?" signal, because `AppContext::from_cli` creates the file as a
/// side effect of opening the connection. The first call always lands here
/// with the file present and an empty projections table.
pub fn run(ctx: &AppContext, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    let path = &ctx.vault_path;
    let projects = ctx.conn.list_projects()?;

    if !projects.is_empty() && !force {
        let count = projects.len();
        return Err(format!(
            "Refusing to initialize: vault at {} already contains {} project(s). \
             Re-run with --force to override.",
            path.display(),
            count
        )
        .into());
    }

    println!("Vault initialized successfully at {}", path.display());
    Ok(())
}
