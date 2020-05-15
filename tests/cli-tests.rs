use std::process::Command;  // Run programs
use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use tempfile::NamedTempFile;
use std::io::Write;
use std::fs;

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("rnc")?;
    cmd.arg("--dos2unix")
        .arg("test/file/doesnt/exist");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("kind: NotFound"));

    Ok(())
}

#[test]
fn find_content_in_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    write!(file, "foo\r\nbar\r\n")?;

    let mut cmd = Command::cargo_bin("rnc")?;
    cmd.arg("--dos2unix")
        .arg(file.path());
    cmd.assert()
        .success();
    let converted = fs::read(file)?;
    assert_eq!(converted, b"foo\nbar\n");

    Ok(())
}