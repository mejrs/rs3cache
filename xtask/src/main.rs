

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
            run_pytests()?;
        },
        Some("fmt") => check_formatting(&cargo)?,
        Some("test") => run_tests(&cargo)?,
        Some("clippy") => run_clippy(&cargo)?,
        Some("pytests") => run_pytests()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "Tasks:
all             All the below
fmt             Test formatting
test            Test with various feature flags
clippy          Run clippy with various feature flags
pytests         Build python extensions and test examples
"
    )
}

fn run_tests(cargo: &str) -> Result<(), DynError> {

    test_with(cargo, &["test", "--features=rs3,mockdata",])?.exit_ok()?;
    test_with(cargo, &["test", "--features=osrs,mockdata"])?.exit_ok()?;
    test_with(cargo, &["test", "--features=legacy,mockdata"])?.exit_ok()?;
    test_with(cargo, &["test", "--features=sqlite", "--manifest-path=rs3cache_backend/Cargo.toml"])?.exit_ok()?;
    test_with(cargo, &["test", "--features=dat2", "--manifest-path=rs3cache_backend/Cargo.toml"])?.exit_ok()?;
    test_with(cargo, &["test", "--features=dat", "--manifest-path=rs3cache_backend/Cargo.toml"])?.exit_ok()?;
    Ok(())
}

fn run_pytests() ->  Result<(), DynError>  {

       let mut command =  Command::new("nox");
       command
            .arg("--non-interactive")
            .arg("-f")
            .arg("rs3_py/noxfile.py");
        println!("Running `nox --non-interactive -f rs3_py/noxfile.py`");
        command
        .status()
        .map_err(|e| format!("failed to execute {:?}", e))?.exit_ok()?;

        let mut command =  Command::new("nox");
       command
            .arg("--non-interactive")
            .arg("-f")
            .arg("osrs_py/noxfile.py");
        println!("Running `nox --non-interactive -f osrs_py/noxfile.py`");
        command
        .status()
        .map_err(|e| format!("failed to execute {:?}", e))?.exit_ok()?;
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

    let mut command_str = String::from("cargo");
    for arg in args{
        command_str.push_str(" ");
        command_str.push_str(arg);
    }
    println!("Running `{command_str}`");
    command
        .status()
        .map_err(|_| format!("failed to execute {:?}", args))
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR")).ancestors().nth(1).unwrap().to_path_buf()
}
