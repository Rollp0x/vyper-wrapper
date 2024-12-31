use std::process::Command;
use std::process::Stdio;
use tempfile::NamedTempFile;
use std::io::Write;
use serde_json::{json, Value};

#[test]
fn test_version_command() {
    let output = Command::new("./target/debug/vyper-wrapper")
        .arg("--version")
        .output()
        .expect("Failed to execute version command");
    
    assert!(output.status.success());
    assert!(!output.stdout.is_empty());
}

#[test]
fn test_blueprint_detection() {
    // Create a temporary contract file with blueprint tag
    let contract_content = r#"
# @blue_print
# pragma version 0.3.10

@external
def test() -> uint256:
    return 42
"#;
    
    let input_json = json!({
        "language": "Vyper",
        "sources": {
            "test.vy": {
                "content": contract_content
            }
        },
        "settings": {
            "outputSelection": {
                "*": ["*"]
            }
        }
    });

    // Execute command with piped input
    let mut cmd = Command::new("./target/debug/vyper-wrapper")
        .arg("--standard-json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    // Write to stdin
    if let Some(mut stdin) = cmd.stdin.take() {
        stdin.write_all(input_json.to_string().as_bytes())
            .expect("Failed to write to stdin");
    }

    // Get output
    let output = cmd.wait_with_output()
        .expect("Failed to read stdout");

    assert!(output.status.success());
    
    // Parse output and verify blueprint bytecode
    let output_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let bytecode = output_json["contracts"]["test.vy"]
        .as_object().unwrap()
        .values().next().unwrap()
        ["evm"]["bytecode"]["object"]
        .as_str().unwrap();

    println!("Blueprint bytecode: {}", bytecode);
    
    // Note: This test uses a specific blueprint pattern for demonstration.
    // In a production environment, you should:
    // 1. Not rely on specific bytecode patterns
    // 2. Use proper blueprint detection mechanisms
    // 3. Consider using contract size difference as a more reliable indicator
    assert!(bytecode.contains("3d81600a3d39f3fe"));  // Example blueprint pattern
}

#[test]
fn test_normal_contract() {
    // Create a temporary contract file without blueprint tag
    let contract_content = r#"
# pragma version 0.3.10

@external
def test() -> uint256:
    return 42
"#;
    
    let input_json = json!({
        "language": "Vyper",
        "sources": {
            "test.vy": {
                "content": contract_content
            }
        },
        "settings": {
            "outputSelection": {
                "*": ["*"]
            }
        }
    });

    // Execute command with piped input
    let mut cmd = Command::new("./target/debug/vyper-wrapper")
        .arg("--standard-json")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    // Write to stdin
    if let Some(mut stdin) = cmd.stdin.take() {
        stdin.write_all(input_json.to_string().as_bytes())
            .expect("Failed to write to stdin");
    }

    // Get output
    let output = cmd.wait_with_output()
        .expect("Failed to read stdout");


    assert!(output.status.success());

    // add error handler
    let output_str = String::from_utf8_lossy(&output.stdout);
    let output_json: Value = serde_json::from_str(&output_str)
        .expect(&format!("Failed to parse JSON: {}", output_str));
    
    // Verify normal bytecode (should not be blueprint format)
    // let output_json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let bytecode = output_json["contracts"]["test.vy"]
        .as_object().unwrap()
        .values().next().unwrap()
        ["evm"]["bytecode"]["object"]
        .as_str().unwrap();

    println!("Normal bytecode: {}", bytecode);

    // Note: The bytecode pattern "3d81600a3d39f3fe" is just an example.
    // The actual blueprint prefix may vary depending on:
    // 1. Vyper version
    // 2. Contract size and content
    // 3. Optimization settings
    // This test is for demonstration purposes only.
    assert!(!bytecode.contains("3d81600a3d39f3fe"));
}

#[test]
fn test_direct_compilation() {
    // Create a temporary contract file
    let contract_content = r#"
@external
def test() -> uint256:
    return 42
"#;
    
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", contract_content).unwrap();

    // Test direct compilation
    let output = Command::new("./target/debug/vyper-wrapper")
        .arg(temp_file.path())
        .output()
        .expect("Failed to execute direct compilation");

    assert!(output.status.success());
}