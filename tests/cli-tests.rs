use assert_cmd::prelude::*; // Add methods on commands
use predicates::prelude::*; // Used for writing assertions
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
    let bin = escargot::CargoBuild::new()
        .bin("rnc")
        .current_release()
        .current_target()
        .run()?;
    let mut cmd = bin.command();
    cmd.arg("--dos2unix").arg("test/file/doesnt/exist");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("kind: NotFound"));

    Ok(())
}

#[test]
fn file_in_place_dos2unix() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    write!(file, "foo\r\nbar\r\n")?;

    let bin = escargot::CargoBuild::new()
        .bin("rnc")
        .current_release()
        .current_target()
        .run()?;
    let mut cmd = bin.command();
    cmd.arg("--dos2unix").arg(file.path());
    cmd.assert().success();
    let converted = fs::read(file)?;
    assert_eq!(converted, b"foo\nbar\n");

    Ok(())
}

#[test]
fn file_in_place_unix2dos() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    write!(file, "foo\nbar\n")?;

    let bin = escargot::CargoBuild::new()
        .bin("rnc")
        .current_release()
        .current_target()
        .run()?;
    let mut cmd = bin.command();
    cmd.arg("--unix2dos").arg(file.path());
    cmd.assert().success();
    let converted = fs::read(file)?;
    assert_eq!(converted, b"foo\r\nbar\r\n");

    Ok(())
}

#[test]
fn new_file_dos2unix() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    let file2 = NamedTempFile::new()?;
    write!(file, "foo\r\nbar\r\n")?;

    let bin = escargot::CargoBuild::new()
        .bin("rnc")
        .current_release()
        .current_target()
        .run()?;
    let mut cmd = bin.command();
    cmd.arg("--dos2unix")
        .arg("--output")
        .arg(file2.path())
        .arg(file.path());
    cmd.assert().success();
    let converted = fs::read(file2)?;
    assert_eq!(converted, b"foo\nbar\n");

    Ok(())
}

#[test]
fn binary_force() -> Result<(), Box<dyn std::error::Error>> {
    let mut file = NamedTempFile::new()?;
    write!(file, "\0\0\r\n\0\0\r\n")?;

    let bin = escargot::CargoBuild::new()
        .bin("rnc")
        .current_release()
        .current_target()
        .run()?;
    let mut cmd = bin.command();
    cmd.arg("--dos2unix");
    cmd.arg("--force");
    cmd.arg(file.path());
    cmd.assert().success();
    let converted = fs::read(file)?;
    assert_eq!(converted, b"\0\0\n\0\0\n");

    Ok(())
}
