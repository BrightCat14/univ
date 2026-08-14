use std::process::{Command, Output};

pub fn run(program: &str, args: &[&str]) -> Result<Output, String> {
    Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run {program}: {e}"))
}

pub fn run_status(program: &str, args: &[&str]) -> Result<(), String> {
    let status = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| format!("failed to run {program}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} exited with {status}"))
    }
}

pub fn run_stdout(program: &str, args: &[&str]) -> Result<String, String> {
    Ok(String::from_utf8_lossy(&run(program, args)?.stdout).into_owned())
}

pub fn command_exists(program: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(program).is_file()))
        .unwrap_or(false)
}

pub fn is_root() -> bool {
    run_stdout("id", &["-u"])
        .ok()
        .map(|u| u.trim() == "0")
        .unwrap_or(false)
}

pub fn run_privileged_status(args: &[&str]) -> Result<(), String> {
    let status = if is_root() {
        Command::new(args[0]).args(&args[1..]).status()
    } else {
        if !command_exists("sudo") {
            return Err("this operation requires root privileges, but sudo was not found".into());
        }
        Command::new("sudo").args(args).status()
    }
    .map_err(|e| format!("failed to run {}: {e}", args[0]))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{} exited with {status}", args.join(" ")))
    }
}
