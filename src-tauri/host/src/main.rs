use interprocess::local_socket::{LocalSocketStream, NameTypeSupport};
use pirate_shared::{DownloadRequest, IpcMessage, IPC_NAME};
use serde_json::Value;
use std::io::{self, BufWriter, Read, Write};
use std::process::Command;
use std::thread;
use std::time::Duration;

fn main() -> io::Result<()> {
    loop {
        // 1. Read message from Chrome (Length-prefixed)
        let mut length_bytes = [0u8; 4];
        if io::stdin().read_exact(&mut length_bytes).is_err() {
            break;
        }
        let length = u32::from_le_bytes(length_bytes) as usize;
        if length == 0 {
            continue;
        }

        let mut buffer = vec![0u8; length];
        io::stdin().read_exact(&mut buffer)?;

        // 2. Parse Message
        let message: Value = match serde_json::from_slice(&buffer) {
            Ok(v) => v,
            Err(e) => {
                send_response(serde_json::json!({ "error": format!("Invalid JSON: {}", e) }))?;
                continue;
            }
        };

        // 3. Extract Payload (Assuming type="DOWNLOAD_REQUEST")
        // In a real scenario, we'd switch on message type.
        // For MVP, we assume the extension sends the payload we need.
        let payload = message.get("payload").unwrap_or(&message).clone();

        let download_req = DownloadRequest {
            url: payload["url"].as_str().unwrap_or("").to_string(),
            filename: payload["filename"].as_str().map(|s| s.to_string()),
            headers: payload["headers"]
                .as_object()
                .map(|h| {
                    h.iter()
                        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            cookies: None, // Extension needs to send this, assume empty for now
            referrer: payload["referrer"].as_str().map(|s| s.to_string()),
        };

        // 4. Send to App via IPC
        match send_to_app(IpcMessage::DownloadRequest(download_req)) {
            Ok(_) => send_response(serde_json::json!({ "status": "sent_to_app" }))?,
            Err(e) => send_response(serde_json::json!({ "error": format!("IPC Failed: {}", e) }))?,
        }
    }
    Ok(())
}

fn send_to_app(msg: IpcMessage) -> io::Result<()> {
    let name = IPC_NAME;

    // Try connecting
    let conn = match LocalSocketStream::connect(name) {
        Ok(c) => c,
        Err(e)
            if e.kind() == io::ErrorKind::ConnectionRefused
                || e.kind() == io::ErrorKind::NotFound =>
        {
            // App not running? Launch it.
            // TODO: Launch logic. For now, just retry once or fail.
            // On Windows, ConnectionRefused usually means pipe doesn't exist.

            // Attempt to launch (Placeholder path - user needs to ensure it's in path or relative)
            // For dev environment, we might not be able to easily launch "cargo run".
            // Let's return error for now and let the user ensure app is open.
            return Err(e);
        }
        Err(e) => return Err(e),
    };

    let mut writer = BufWriter::new(conn);

    // Serialize and write to pipe (Newline delimited JSON for simplicity in shared stream)
    let json =
        serde_json::to_string(&msg).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    writer.write_all(json.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;

    Ok(())
}

fn send_response(value: Value) -> io::Result<()> {
    let bytes = serde_json::to_vec(&value)?;
    let len = bytes.len() as u32;
    io::stdout().write_all(&len.to_le_bytes())?;
    io::stdout().write_all(&bytes)?;
    io::stdout().flush()?;
    Ok(())
}
