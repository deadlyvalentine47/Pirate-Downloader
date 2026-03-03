use tracing::{debug, warn};

pub struct StreamProcessor {
    enable_header_stripping: bool,
}

impl StreamProcessor {
    pub fn new(enable_header_stripping: bool) -> Self {
        Self { enable_header_stripping }
    }

    /// Cleans the segment by searching for the first occurrence of the MPEG-TS 
    /// sync byte (0x47) within the first 1024 bytes and stripping everything before it.
    pub fn clean_segment(&self, bytes: Vec<u8>) -> Vec<u8> {
        if !self.enable_header_stripping {
            return bytes;
        }

        // Check if it already starts with the sync byte (most common case).
        if bytes.is_empty() || bytes[0] == 0x47 {
            return bytes;
        }

        // Search for the sync byte in the first 1024 bytes.
        let search_limit = std::cmp::min(bytes.len(), 1024);
        let mut sync_offset = None;

        for i in 0..search_limit {
            if bytes[i] == 0x47 {
                sync_offset = Some(i);
                break;
            }
        }

        if let Some(offset) = sync_offset {
            debug!("Stripped {} bytes of junk header from segment", offset);
            bytes[offset..].to_vec()
        } else {
            // If we didn't find the sync byte, it might not be a standard MPEG-TS segment.
            // In this case, we return it as-is instead of discarding it.
            warn!("MPEG-TS sync byte (0x47) not found in the first 1024 bytes of the segment.");
            bytes
        }
    }
}
