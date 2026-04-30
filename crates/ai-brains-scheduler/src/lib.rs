use thiserror::Error;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Render error: {0}")]
    RenderError(String),
}

pub type Result<T> = std::result::Result<T, SchedulerError>;

pub struct TaskScheduler;

impl TaskScheduler {
    /// Renders a Windows schtasks command to run the nightly job.
    /// exe_path: Full path to the ai-brains.exe
    /// task_name: Unique name for the task (e.g. "AI-Brains-Nightly")
    /// start_time: Format "HH:mm" (e.g. "03:00")
    pub fn render_create_command(exe_path: &str, task_name: &str, start_time: &str) -> String {
        // We use single quotes around the path to handle spaces in Windows paths,
        // as per schtasks requirements.
        format!(
            "schtasks /create /tn \"{}\" /tr \"'{}' nightly\" /sc daily /st {} /f",
            task_name, exe_path, start_time
        )
    }

    pub fn render_delete_command(task_name: &str) -> String {
        format!("schtasks /delete /tn \"{}\" /f", task_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_create_command() {
        let cmd = TaskScheduler::render_create_command(
            r"C:\Program Files\AI-Brains\ai-brains.exe",
            "AI-Brains-Nightly",
            "03:00",
        );
        assert_eq!(
            cmd,
            r#"schtasks /create /tn "AI-Brains-Nightly" /tr "'C:\Program Files\AI-Brains\ai-brains.exe' nightly" /sc daily /st 03:00 /f"#
        );
    }
}
