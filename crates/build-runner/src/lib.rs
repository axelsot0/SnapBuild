use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunResult {
    pub command: String,
    pub exit_code: i32,
    pub duration_ms: u128,
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
pub enum RunnerError {
    Io(std::io::Error),
}

impl From<std::io::Error> for RunnerError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub fn run_command(cwd: &Path, command: &str, args: &[&str]) -> Result<RunResult, RunnerError> {
    let started = std::time::Instant::now();
    let output = Command::new(command).args(args).current_dir(cwd).output()?;
    let duration_ms = started.elapsed().as_millis();

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(RunResult {
        command: std::iter::once(command)
            .chain(args.iter().copied())
            .collect::<Vec<_>>()
            .join(" "),
        exit_code,
        duration_ms,
        success: output.status.success(),
        stdout,
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_simple_command() {
        let cwd = std::env::temp_dir();
        let result = run_command(&cwd, "sh", &["-c", "echo snapbuild"]).unwrap();
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("snapbuild"));
    }
}
