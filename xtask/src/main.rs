

#![feature(exit_status_error)]

use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

type DynError = Box<dyn std::error::Error>;

fn main() -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());

    let task = env::args().nth(1);
    match task.as_ref().map(|it| it.as_str()) {
        Some("all") => {
            check_formatting(&cargo)?;
            run_clippy(&cargo)?;
            run_tests(&cargo)?;
        },
        Some("fmt") => check_formatting(&cargo)?,
        Some("test") => run_tests(&cargo)?,
        Some("clippy") => run_clippy(&cargo)?,
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

fn run_tests(cargo: &str) -> Result<(), DynError> {

    test_with(cargo, &["test", "--features=rs3,mockdata"])?.exit_ok()?;
    test_with(cargo, &["test", "--features=osrs,mockdata"])?.exit_ok()?;
    test_with(cargo, &["test", "--features=legacy,mockdata"])?.exit_ok()?;
   

    Ok(())
}

fn check_formatting(cargo: &str) -> Result<(), DynError>{
    test_with(cargo, &["fmt", "--all", "--", "--check"])?.exit_ok()?;
    Ok(())
}

fn run_clippy(cargo: &str) -> Result<(), DynError> {
    

    test_with(cargo, &["clippy", "--features=rs3,pyo3", "--", "-D", "warnings"])?.exit_ok()?;
    test_with(cargo, &["clippy", "--features=osrs,pyo3", "--", "-D", "warnings"])?.exit_ok()?;
    test_with(cargo, &["clippy", "--features=legacy,pyo3", "--", "-D", "warnings"])?.exit_ok()?;
    Ok(())
}

fn test_with(cargo: &str, args: &[&str]) -> Result<ExitStatus, String> {
    let mut command = Command::new(cargo);
    command.current_dir(project_root());
    command.args(args);

    println!("Running {command:?}");
    command
        .status()
        .map_err(|_| format!("failed to execute {:?}", args))
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR")).ancestors().nth(1).unwrap().to_path_buf()
}
