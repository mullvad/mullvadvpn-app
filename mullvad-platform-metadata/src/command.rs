use std::process::Command;

/// Helper for getting stdout of some command as a String. Ignores the exit code of the command.
pub fn command_stdout_lossy(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .ok()
}
