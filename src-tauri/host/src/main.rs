use serde_json::Value;
use std::io::{self, Read, Write};

fn main() -> io::Result<()> {
    loop {
        // 1. Read the message length (4 bytes, little-endian)
        let mut length_bytes = [0u8; 4];
        if let Err(_) = io::stdin().read_exact(&mut length_bytes) {
            // EOF or error, exit loop
            break;
        }
        let length = u32::from_le_bytes(length_bytes) as usize;

        if length == 0 {
            continue;
        }

        // 2. Read the message content
        let mut buffer = vec![0u8; length];
        io::stdin().read_exact(&mut buffer)?;

        // 3. Parse and Process (Echo for now)
        // For debugging, we just treat it as a Value
        let message: Value = match serde_json::from_slice(&buffer) {
            Ok(v) => v,
            Err(_) => {
                // If invalid JSON, maybe log error? For now just skip
                continue;
            }
        };

        // 4. Send Response (Echo)
        // For MVP test: { "echo": <received_message> }
        let response = serde_json::json!({
            "echo": message
        });

        let response_bytes = serde_json::to_vec(&response)?;
        let response_len = response_bytes.len() as u32;

        // Write length
        io::stdout().write_all(&response_len.to_le_bytes())?;
        // Write content
        io::stdout().write_all(&response_bytes)?;
        io::stdout().flush()?;
    }

    Ok(())
}
