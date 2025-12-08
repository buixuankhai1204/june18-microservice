use crate::core::configure::app::get_static_dir;
use crate::core::error::{AppError, AppResult};
use axum_extra::extract::Multipart;
use std::path::Path;
use tokio::{fs, io::AsyncWriteExt};

pub async fn store_file<P: AsRef<Path>>(file_path: &P, content: &[u8]) -> AppResult<()> {
    if let Some(parent_dir) = file_path.as_ref().parent() {
        fs::create_dir_all(&parent_dir).await?;
    }

    let mut file = fs::File::create(&file_path).await?;
    file.write_all(content).await?;
    Ok(())
}

pub async fn save_file(file: &mut Multipart) -> Result<String, AppError> {
    let mut file_name = String::new();

    while let Some(field) = file
        .next_field()
        .await
        .map_err(|e| AppError::BadRequestError(format!("Failed to read field: {}", e)))?
    {
        if let Some(file_name_value) = field.file_name().map(|f| f.to_string()) {
            let data = field.bytes().await.map_err(|e| {
                AppError::BadRequestError(format!("Failed to read file bytes: {}", e))
            })?;

            let name_uuid = uuid::Uuid::new_v4();
            file_name = format!(
                "{}.{}",
                name_uuid.to_string(),
                file_name_value.split('.').last().unwrap_or("")
            );

            let images_dir = get_static_dir().unwrap().join("images");
            tokio::fs::create_dir_all(&images_dir).await.map_err(|e| {
                AppError::BadRequestError(format!("Failed to create images directory: {}", e))
            })?;

            tokio::fs::write(&images_dir.join(file_name.as_str()), &data)
                .await
                .map_err(|e| AppError::BadRequestError(format!("Failed to save file: {}", e)))?;
        } else {
            return Err(AppError::BadRequestError("Missing field name or file name".to_string()));
        }
    }

    Ok(file_name)
}
