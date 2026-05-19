mod alias;
mod canonical;
mod discovery;
mod display;
mod errors;
mod project_path;
mod symlink;
mod unc;
mod windows;
mod wsl;

pub use canonical::normalize_project_path;
pub use discovery::{extract_project_id_from_changeguard, find_changeguard_dir};
pub use display::display_path;
pub use errors::{PathError, Result};
pub use project_path::ProjectPath;
