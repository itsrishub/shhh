use base64::{engine::general_purpose::STANDARD, Engine};
use std::io::{self, Read};
use std::process;

fn decode_base64(encoded: &str) -> String {
    let cleaned: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
    
    match STANDARD.decode(&cleaned) {
        Ok(bytes) => {
            String::from_utf8(bytes.clone())
                .unwrap_or_else(|_| String::from_utf8_lossy(&bytes).to_string())
        }
        Err(_) => encoded.to_string(),
    }
}

fn print_usage_and_exit() -> ! {
    eprintln!("Usage: kubectl get secret <name> -o yaml | shhh");
    eprintln!();
    eprintln!("shhh decodes base64-encoded data from Kubernetes secrets.");
    eprintln!("Pipe the YAML output of 'kubectl get secret' to this tool.");
    process::exit(1);
}

fn main() {
    // Check if stdin is a terminal (no pipe)
    if atty::is(atty::Stream::Stdin) {
        eprintln!("Error: No input provided.");
        print_usage_and_exit();
    }

    let mut input = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("Error reading stdin: {}", e);
        process::exit(1);
    }

    if input.trim().is_empty() {
        eprintln!("Error: Empty input.");
        print_usage_and_exit();
    }

    // Validate that this looks like a Kubernetes secret
    let has_kind_secret = input.lines().any(|l| {
        let t = l.trim();
        t == "kind: Secret" || t.starts_with("kind: Secret ")
    });
    
    let has_data_section = input.lines().any(|l| l.trim_start().starts_with("data:"));
    
    if !has_kind_secret {
        eprintln!("Error: Input does not appear to be a Kubernetes Secret.");
        eprintln!("       Expected 'kind: Secret' in the YAML.");
        print_usage_and_exit();
    }

    if !has_data_section {
        eprintln!("Error: No 'data:' section found in the Secret.");
        eprintln!("       The secret may be empty or use 'stringData:' only.");
        print_usage_and_exit();
    }

    let mut in_data_section = false;
    let mut data_indent: Option<usize> = None;

    for line in input.lines() {
        let trimmed = line.trim_start();
        let current_indent = line.len() - trimmed.len();

        // Detect entering/leaving the data: section
        if trimmed.starts_with("data:") && !trimmed.starts_with("stringData:") {
            in_data_section = true;
            data_indent = None;
            println!("{}", line);
            continue;
        }

        // If we're in data section, check if we've left it
        if in_data_section {
            if let Some(base_indent) = data_indent {
                if current_indent <= base_indent && !trimmed.is_empty() && !trimmed.starts_with('#') {
                    if current_indent == 0 || (current_indent < base_indent) {
                        in_data_section = false;
                        data_indent = None;
                    }
                }
            }
        }

        // Process lines in data section
        if in_data_section && trimmed.contains(':') && !trimmed.starts_with('#') {
            if data_indent.is_none() {
                data_indent = Some(current_indent);
            }

            if let Some(base) = data_indent {
                if current_indent == base {
                    if let Some(colon_pos) = trimmed.find(':') {
                        let key = &trimmed[..colon_pos];
                        let value = trimmed[colon_pos + 1..].trim();
                        
                        if !value.is_empty() {
                            let decoded = decode_base64(value);
                            let indent = &line[..current_indent];
                            println!("{}{}: {}", indent, key, decoded);
                            continue;
                        }
                    }
                }
            }
        }

        println!("{}", line);
    }
}
