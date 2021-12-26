use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

type DynError = Box<dyn std::error::Error>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    match task.as_ref().map(|it| it.as_str()) {
        Some("test") => test_binaries()?,
        Some("clippy") => test_clippy()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:
test            Test with various feature flags
"
    )
}

fn test_binaries() -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    test_with(&cargo, &["test", "--features", "rs3"])?;
    test_with(&cargo, &["test", "--features", "osrs"])?;

    Ok(())
}

fn test_clippy() -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    test_with(&cargo, &["clippy", "--features", "rs3,python", "--", "-D", "warnings"])?;
    test_with(&cargo, &["clippy", "--features", "osrs,python", "--", "-D", "warnings"])?;

    Ok(())
}

fn test_with(cargo: &str, args: &[&str]) -> Result<ExitStatus, String> {
    Command::new(cargo)
        .current_dir(project_root())
        .args(args)
        .status()
        .map_err(|_| format!("failed to execute {:?}", args))
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR")).ancestors().nth(1).unwrap().to_path_buf()
}
