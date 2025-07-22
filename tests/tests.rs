use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let path = env::temp_dir().join(name);
        if path.exists() {
            fs::remove_dir_all(&path).expect("Failed to clean up pre-existing test directory");
        }
        fs::create_dir_all(&path).expect("Failed to create test directory");
        TestDir { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path).expect("Failed to clean up test directory on drop");
    }
}

#[test]
fn test_directory_bundling_with_all_ignores() {
    let test_dir = TestDir::new("fcat_test_dir");
    let output_file = env::temp_dir().join("fcat_test_dir_output.txt");

    let subdir = test_dir.path().join("src");
    fs::create_dir(&subdir).unwrap();
    fs::write(test_dir.path().join("file_a.txt"), "content a").unwrap();
    fs::write(subdir.join("file_b.rs"), "content b").unwrap();

    let git_dir = test_dir.path().join(".git");
    fs::create_dir(&git_dir).unwrap();
    fs::write(git_dir.join("config"), "git content").unwrap();

    let excluded_dir = test_dir.path().join("a_target_like_dir");
    fs::create_dir(&excluded_dir).unwrap();
    fs::write(excluded_dir.join("ignored.txt"), "ignored content").unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_fcat"))
        .args([
            test_dir.path().to_str().unwrap(),
            "--output-file",
            output_file.to_str().unwrap(),
            "--exclude-dir",
            excluded_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute fcat command");

    assert!(status.success(), "fcat command did not exit successfully");
    assert!(output_file.exists(), "Output file was not created");

    let output_content = fs::read_to_string(&output_file).unwrap();

    assert!(
        output_content.contains("content a"),
        "Content of file_a.txt is missing"
    );
    assert!(
        output_content.contains("content b"),
        "Content of file_b.rs is missing"
    );
    assert!(
        !output_content.contains("git content"),
        "Content from .git directory was not ignored"
    );
    assert!(
        !output_content.contains("ignored content"),
        "Content from user-excluded directory was not ignored"
    );

    fs::remove_file(output_file).unwrap();
}

#[test]
fn test_single_file_bundling() {
    let test_dir = TestDir::new("fcat_test_single_file");
    let output_file = env::temp_dir().join("fcat_test_single_output.txt");

    let target_file = test_dir.path().join("file_a.txt");
    fs::write(&target_file, "single file content").unwrap();
    fs::write(test_dir.path().join("file_b.txt"), "other content").unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_fcat"))
        .args([
            target_file.to_str().unwrap(),
            "--output-file",
            output_file.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute fcat command");

    assert!(status.success(), "fcat command did not exit successfully");
    assert!(output_file.exists(), "Output file was not created");

    let output_content = fs::read_to_string(&output_file).unwrap();

    assert!(
        output_content.contains("single file content"),
        "Content of the target file is missing"
    );
    assert!(
        !output_content.contains("other content"),
        "Content from non-target file was incorrectly included"
    );

    fs::remove_file(output_file).unwrap();
}

#[test]
fn test_multiple_paths_bundling() {
    let test_dir = TestDir::new("fcat_test_multiple_paths");
    let output_file = env::temp_dir().join("fcat_test_multiple_output.txt");

    let file_a = test_dir.path().join("file_a.txt");
    fs::write(&file_a, "content of file A").unwrap();

    let subdir = test_dir.path().join("subdir");
    fs::create_dir(&subdir).unwrap();
    let file_b = subdir.join("file_b.txt");
    fs::write(&file_b, "content of file B").unwrap();

    let file_c = test_dir.path().join("file_c.txt");
    fs::write(&file_c, "content of file C (should be ignored)").unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_fcat"))
        .args([
            file_a.to_str().unwrap(),
            file_b.to_str().unwrap(),
            "--output-file",
            output_file.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute fcat command");

    assert!(status.success(), "fcat command did not exit successfully");
    assert!(output_file.exists(), "Output file was not created");

    let output_content = fs::read_to_string(&output_file).unwrap();

    assert!(
        output_content.contains("content of file A"),
        "Content of file_a.txt is missing"
    );
    assert!(
        output_content.contains("content of file B"),
        "Content of file_b.txt is missing"
    );
    assert!(
        !output_content.contains("content of file C"),
        "Content from the ignored file_c.txt was incorrectly included"
    );

    fs::remove_file(output_file).unwrap();
}

#[test]
fn test_default_ignores_target_directory() {
    let test_dir = TestDir::new("fcat_test_target_ignore");
    let output_file = env::temp_dir().join("fcat_test_target_output.txt");

    let src_dir = test_dir.path().join("src");
    fs::create_dir(&src_dir).unwrap();
    fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

    let target_dir = test_dir.path().join("target");
    fs::create_dir_all(&target_dir).unwrap();
    let target_subdir = target_dir.join("debug");
    fs::create_dir_all(&target_subdir).unwrap();
    fs::write(
        target_subdir.join("some_file.rs"),
        "some rust file in target",
    )
    .unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_fcat"))
        .args([
            test_dir.path().to_str().unwrap(),
            "--output-file",
            output_file.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute fcat command");

    assert!(status.success(), "fcat command did not exit successfully");
    assert!(output_file.exists(), "Output file was not created");

    let output_content = fs::read_to_string(&output_file).unwrap();

    assert!(
        output_content.contains("fn main() {}"),
        "Content of src/main.rs is missing"
    );
    assert!(
        !output_content.contains("some rust file in target"),
        "Content from target directory was not ignored"
    );

    fs::remove_file(output_file).unwrap();
}
