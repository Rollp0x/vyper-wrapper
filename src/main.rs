use serde_json::{json, Value};
use std::env;
use std::{
    io::{self, Read, Write},
    process::{Command, Stdio},
};
use tempfile::NamedTempFile;

/// Get blueprint bytecode for a contract by calling vyper_path with blueprint_bytecode format
fn get_blueprint_bytecode(
    content: &str,
    vyper_path: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut temp_file = NamedTempFile::new()?;
    temp_file.write_all(content.as_bytes())?;

    let output = Command::new(vyper_path)
        .arg("-f")
        .arg("blueprint_bytecode")
        .arg(temp_file.path())
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    } else {
        Err("Failed to get blueprint bytecode".into())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let vyper_path = env::var("VYPER_ORIGIN_PATH").unwrap_or_else(|_| "vyper.origin".to_string());

    match args.get(1).map(|s| s.as_str()) {
        Some("--version") => {
            let output = Command::new(&vyper_path).arg("--version").output()?;
            io::stdout().write_all(&output.stdout)?;
        }

        Some("--standard-json") => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let input_json: Value = serde_json::from_str(&input)?;

            let mut blueprint_files = Vec::new();
            if let Some(sources) = input_json.get("sources").and_then(|s| s.as_object()) {
                for (filename, source) in sources {
                    if let Some(content) = source.get("content").and_then(|c| c.as_str()) {
                        if content.contains("@blue_print") {
                            blueprint_files.push((filename.clone(), content.to_string()));
                        }
                    }
                }
            }

            let mut cmd = Command::new(&vyper_path)
                .arg("--standard-json")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            if let Some(mut stdin) = cmd.stdin.take() {
                stdin.write_all(input.as_bytes())?;
            }

            let output = cmd.wait_with_output()?;
            let mut output_json: Value = serde_json::from_str(&String::from_utf8(output.stdout)?)?;

            for (filename, content) in blueprint_files {
                if let Ok(blueprint_code) = get_blueprint_bytecode(&content, &vyper_path) {
                    if let Some(contracts) = output_json
                        .get_mut("contracts")
                        .and_then(|c| c.as_object_mut())
                    {
                        if let Some(file_contracts) =
                            contracts.get_mut(&filename).and_then(|f| f.as_object_mut())
                        {
                            if let Some((_, contract)) = file_contracts.iter_mut().next() {
                                if let Some(evm) =
                                    contract.get_mut("evm").and_then(|e| e.as_object_mut())
                                {
                                    if let Some(bytecode) =
                                        evm.get_mut("bytecode").and_then(|b| b.as_object_mut())
                                    {
                                        bytecode
                                            .insert("object".to_string(), json!(blueprint_code));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let output_str = serde_json::to_string(&output_json)?;
            io::stdout().write_all(output_str.as_bytes())?;
        }

        Some("-f") if args.get(2).map_or(false, |s| s == "combined_json") => {
            let mut cmd = Command::new(&vyper_path);
            for arg in args.iter().skip(1) {
                cmd.arg(arg);
            }

            let output = cmd.output()?;
            if !output.status.success() {
                io::stderr().write_all(&output.stderr)?;
                std::process::exit(output.status.code().unwrap_or(1));
            }

            let mut output_json: Value =
                serde_json::from_str(&String::from_utf8_lossy(&output.stdout))?;

            for source_path in args.iter().skip(3) {
                let content = std::fs::read_to_string(source_path)?;
                if content.contains("@blue_print") {
                    let path = std::path::Path::new(source_path);
                    let file_name = path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(source_path);

                    let possible_keys = vec![
                        file_name.to_string(),
                        path.components()
                            .skip_while(|c| c.as_os_str() != "contracts")
                            .collect::<std::path::PathBuf>()
                            .to_string_lossy()
                            .to_string(),
                    ];

                    if let Ok(blueprint_code) = get_blueprint_bytecode(&content, &vyper_path) {
                        if let Some(obj) = output_json.as_object_mut() {
                            for key in possible_keys {
                                if let Some(contract_obj) = obj.get_mut(&key) {
                                    if let Some(contract_map) = contract_obj.as_object_mut() {
                                        contract_map
                                            .insert("bytecode".to_string(), json!(blueprint_code));
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let output_str = serde_json::to_string(&output_json)?;
            io::stdout().write_all(output_str.as_bytes())?;
        }

        _ => {
            let mut cmd = Command::new(&vyper_path);
            for arg in args.iter().skip(1) {
                cmd.arg(arg);
            }

            let output = cmd.output()?;
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;

            if !output.status.success() {
                std::process::exit(output.status.code().unwrap_or(1));
            }
        }
    }

    Ok(())
}
