use std::{io::{self, Read, Write}, process::{Command, Stdio}};
use serde_json::{Value, json};
use tempfile::NamedTempFile;
use std::env;

/// Get blueprint bytecode for a contract by calling vyper_path with blueprint_bytecode format
fn get_blueprint_bytecode(content: &str,vyper_path:&str) -> Result<String, Box<dyn std::error::Error>> {
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

    // get path of vyper.origin 
    let vyper_path = if let Ok(path) = env::var("VYPER_ORIGIN_PATH") {
        path
    } else {
        "vyper.origin".to_string()  // default value
    };
    
    match args.get(1).map(|s| s.as_str()) {
        Some("--version") => {
            let output = Command::new(&vyper_path)
                .arg("--version")
                .output()?;
            io::stdout().write_all(&output.stdout)?;
        }
        
        Some("--standard-json") => {
            // 1. Read JSON input from stdin
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let input_json: Value = serde_json::from_str(&input)?;
            
            // 2. Check each source file for @blue_print tag
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
            
            // 3. Call standard compilation
            let mut cmd = Command::new(&vyper_path)
                .arg("--standard-json")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;
                
            if let Some(mut stdin) = cmd.stdin.take() {
                stdin.write_all(input.as_bytes())?;
            }
            
            let output = cmd.wait_with_output()?;
            let mut output_json: Value = serde_json::from_str(
                &String::from_utf8(output.stdout)?
            )?;
            
            // 4. Replace bytecode for blueprint contracts
            for (filename, content) in blueprint_files {
                if let Ok(blueprint_code) = get_blueprint_bytecode(&content,&vyper_path) {
                    if let Some(contracts) = output_json.get_mut("contracts").and_then(|c| c.as_object_mut()) {
                        if let Some(file_contracts) = contracts.get_mut(&filename).and_then(|f| f.as_object_mut()) {
                            if let Some((_, contract)) = file_contracts.iter_mut().next() {
                                if let Some(evm) = contract.get_mut("evm").and_then(|e| e.as_object_mut()) {
                                    if let Some(bytecode) = evm.get_mut("bytecode").and_then(|b| b.as_object_mut()) {
                                        bytecode.insert("object".to_string(), json!(blueprint_code));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // 5. Output final result
            let output_str = serde_json::to_string(&output_json)?;
            io::stdout().write_all(output_str.as_bytes())?;
        }
        
        _ => {
            // Forward all arguments to original vyper
            let mut cmd = Command::new(&vyper_path);
            
            // Skip program name, add all remaining arguments
            for arg in args.iter().skip(1) {
                cmd.arg(arg);
            }
            
            // Execute command and forward all output
            let output = cmd.output()?;
            
            // Write to stdout
            io::stdout().write_all(&output.stdout)?;
            
            // Write to stderr
            io::stderr().write_all(&output.stderr)?;
            
            // Maintain original exit status
            if !output.status.success() {
                std::process::exit(output.status.code().unwrap_or(1));
            }
        }
    }
    
    Ok(())
}

