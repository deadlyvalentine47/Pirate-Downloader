/// File system utilities for download management
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

/// Allocates a sparse file of the given size
///
/// Creates a file and seeks to the end, writing a single byte.
/// This reserves disk space without actually writing zeros to the entire file.
///
/// # Arguments
/// * `path` - Path where the file should be created
/// * `size` - Total size of the file in bytes
///
/// # Returns
/// `Ok(())` if successful
///
/// # Errors
/// Returns `std::io::Error` if:
/// - File cannot be created
/// - Seek operation fails
/// - Write operation fails
pub fn allocate_sparse_file(path: &Path, size: u64) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    file.seek(SeekFrom::Start(size - 1))?;
    file.write_all(&[0])?;
    Ok(())
}

/// Calculates optimal chunk size based on total file size
///
/// Uses tiered strategy:
/// - < 100 MB: 512 KB chunks (fast completion)
/// - 100 MB - 1 GB: 4 MB chunks (balanced)
/// - 1 GB - 10 GB: 16 MB chunks (fewer requests)
/// - > 10 GB: 64 MB chunks (minimal overhead)
///
/// # Arguments
/// * `total_size` - Total file size in bytes
///
/// # Returns
/// Optimal chunk size in bytes
pub fn calculate_chunk_size(total_size: u64) -> u64 {
    if total_size < 100 * 1024 * 1024 {
        512 * 1024 // 512 KB for < 100 MB
    } else if total_size < 1 * 1024 * 1024 * 1024 {
        4 * 1024 * 1024 // 4 MB for 100 MB - 1 GB
    } else if total_size < 10 * 1024 * 1024 * 1024 {
        16 * 1024 * 1024 // 16 MB for 1 GB - 10 GB
    } else {
        64 * 1024 * 1024 // 64 MB for > 10 GB
    }
}
