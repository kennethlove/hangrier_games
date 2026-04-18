use crate::AppError;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// File validation constraints
pub struct UploadConstraints {
    pub max_size_bytes: usize,
    pub allowed_extensions: Vec<String>,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

impl Default for UploadConstraints {
    fn default() -> Self {
        Self {
            max_size_bytes: 5 * 1024 * 1024, // 5MB
            allowed_extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "webp".to_string(),
            ],
            max_width: Some(1024),
            max_height: Some(1024),
        }
    }
}

/// Storage backend trait - allows swapping between local filesystem and S3/MinIO
#[async_trait::async_trait]
pub trait StorageBackend: Send + Sync {
    /// Save a file and return its public URL path
    async fn save(&self, path: &str, data: &[u8]) -> Result<String, AppError>;

    /// Delete a file
    async fn delete(&self, path: &str) -> Result<(), AppError>;

    /// Check if a file exists
    async fn exists(&self, path: &str) -> Result<bool, AppError>;

    /// Get the public URL for accessing the file
    fn public_url(&self, path: &str) -> String;
}

/// Local filesystem storage implementation
pub struct LocalStorage {
    base_path: PathBuf,
    public_prefix: String,
}

impl LocalStorage {
    pub fn new(base_path: impl AsRef<Path>, public_prefix: impl Into<String>) -> Self {
        Self {
            base_path: base_path.as_ref().to_path_buf(),
            public_prefix: public_prefix.into(),
        }
    }

    /// Ensure the storage directory exists
    pub async fn init(&self) -> Result<(), AppError> {
        fs::create_dir_all(&self.base_path).await.map_err(|e| {
            AppError::InternalServerError(format!("Failed to create storage directory: {}", e))
        })
    }
}

#[async_trait::async_trait]
impl StorageBackend for LocalStorage {
    async fn save(&self, path: &str, data: &[u8]) -> Result<String, AppError> {
        let file_path = self.base_path.join(path);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to create directory: {}", e))
            })?;
        }

        // Write file
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to create file: {}", e)))?;

        file.write_all(data)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to write file: {}", e)))?;

        Ok(path.to_string())
    }

    async fn delete(&self, path: &str) -> Result<(), AppError> {
        let file_path = self.base_path.join(path);

        if file_path.exists() {
            fs::remove_file(&file_path).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to delete file: {}", e))
            })?;
        }

        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool, AppError> {
        let file_path = self.base_path.join(path);
        Ok(file_path.exists())
    }

    fn public_url(&self, path: &str) -> String {
        format!("{}/{}", self.public_prefix, path)
    }
}

/// Validate uploaded file data
pub fn validate_upload(
    data: &[u8],
    filename: &str,
    constraints: &UploadConstraints,
) -> Result<(), AppError> {
    // Check size
    if data.len() > constraints.max_size_bytes {
        return Err(AppError::ValidationError(format!(
            "File size {} bytes exceeds maximum {} bytes",
            data.len(),
            constraints.max_size_bytes
        )));
    }

    // Check extension
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .ok_or_else(|| AppError::ValidationError("File has no extension".to_string()))?;

    if !constraints.allowed_extensions.contains(&ext) {
        return Err(AppError::ValidationError(format!(
            "File extension '{}' not allowed. Allowed: {}",
            ext,
            constraints.allowed_extensions.join(", ")
        )));
    }

    // Basic image format validation (magic bytes)
    validate_image_format(data, &ext)?;

    Ok(())
}

/// Validate image file format by checking magic bytes
fn validate_image_format(data: &[u8], expected_ext: &str) -> Result<(), AppError> {
    if data.len() < 12 {
        return Err(AppError::ValidationError(
            "File too small to be valid image".to_string(),
        ));
    }

    let is_valid = match expected_ext {
        "jpg" | "jpeg" => data.starts_with(&[0xFF, 0xD8, 0xFF]),
        "png" => data.starts_with(&[0x89, 0x50, 0x4E, 0x47]),
        "webp" => data.starts_with(b"RIFF") && data[8..12] == *b"WEBP",
        _ => {
            return Err(AppError::ValidationError(format!(
                "Unknown image format: {}",
                expected_ext
            )));
        }
    };

    if !is_valid {
        return Err(AppError::ValidationError(format!(
            "File content does not match {} format",
            expected_ext
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_jpeg_magic_bytes() {
        let jpeg_data = vec![
            0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
        ];
        assert!(validate_image_format(&jpeg_data, "jpg").is_ok());
    }

    #[test]
    fn test_validate_png_magic_bytes() {
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
        ];
        assert!(validate_image_format(&png_data, "png").is_ok());
    }

    #[test]
    fn test_reject_invalid_magic_bytes() {
        let invalid_data = vec![
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        assert!(validate_image_format(&invalid_data, "jpg").is_err());
    }

    #[test]
    fn test_validate_upload_size() {
        let constraints = UploadConstraints {
            max_size_bytes: 100,
            ..Default::default()
        };
        let large_data = vec![0xFF; 200];
        assert!(validate_upload(&large_data, "test.jpg", &constraints).is_err());
    }

    #[test]
    fn test_validate_upload_extension() {
        let constraints = UploadConstraints::default();
        let data = vec![0xFF; 50];
        assert!(validate_upload(&data, "test.exe", &constraints).is_err());
    }
}
