// src/api/handlers/upload.rs
use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use google_cloud_storage::{
    client::{Client, ClientConfig},
    http::{
        object_access_controls::PredefinedObjectAcl,
        objects::{
            Object,
            upload::{UploadObjectRequest, UploadType},
        },
    },
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub url: String,
    pub filename: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// 處理圖片上傳到 Firebase Storage
pub async fn upload_image(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    // 從 multipart 中提取文件
    let mut file_data: Option<Vec<u8>> = None;
    let mut original_filename: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Failed to read multipart field: {}", e),
            }),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "image" || name == "file" {
            original_filename = field.file_name().map(|s| s.to_string());
            content_type = field.content_type().map(|s| s.to_string());

            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::BAD_REQUEST,
                            Json(ErrorResponse {
                                error: format!("Failed to read file data: {}", e),
                            }),
                        )
                    })?
                    .to_vec(),
            );
        }
    }

    // 驗證文件是否存在
    let data = file_data.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No file provided".to_string(),
            }),
        )
    })?;

    // 驗證文件大小 (例如限制 10MB)
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    if data.len() > MAX_FILE_SIZE {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ErrorResponse {
                error: "File size exceeds 10MB limit".to_string(),
            }),
        ));
    }

    // 驗證文件類型
    let mime_type = content_type.clone().unwrap_or_else(|| {
        mime_guess::from_path(original_filename.as_ref().unwrap_or(&String::new()))
            .first_or_octet_stream()
            .to_string()
    });

    if !mime_type.starts_with("image/") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Only image files are allowed".to_string(),
            }),
        ));
    }

    // 生成唯一文件名
    let extension = original_filename
        .as_ref()
        .and_then(|name| name.split('.').last())
        .unwrap_or("jpg");

    let unique_filename = format!("axum-app-uploads/{}.{}", Uuid::new_v4(), extension);
    let file_size = data.len() as u64;

    // 上傳到 Firebase Storage
    let url = upload_to_firebase(&state, &unique_filename, data, &mime_type)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to upload to Firebase: {}", e),
                }),
            )
        })?;

    Ok(Json(UploadResponse {
        url,
        filename: unique_filename,
        size: file_size,
    }))
}

/// 上傳文件到 Firebase Storage（使用 Multipart 上傳以設置完整 metadata）
async fn upload_to_firebase(
    _state: &AppState,
    filename: &str,
    data: Vec<u8>,
    content_type: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // 從環境變數或配置中獲取 Firebase 設定
    let bucket_name =
        std::env::var("FIREBASE_STORAGE_BUCKET").expect("FIREBASE_STORAGE_BUCKET must be set");

    // 創建 Google Cloud Storage 客戶端
    let config = ClientConfig::default().with_auth().await?;

    let client = Client::new(config);

    // 生成 Firebase download token
    let download_token = Uuid::new_v4().to_string();

    // 創建 metadata
    let mut metadata = std::collections::HashMap::new();
    metadata.insert(
        "firebaseStorageDownloadTokens".to_string(),
        download_token.clone(),
    );

    // 創建 Object metadata
    let object = Object {
        name: filename.to_string(),
        bucket: bucket_name.clone(),
        content_type: Some(content_type.to_string()),
        cache_control: Some("public, max-age=31536000".to_string()),
        metadata: Some(metadata),
        ..Default::default()
    };

    // 準備上傳請求
    let upload_request = UploadObjectRequest {
        bucket: bucket_name.clone(),
        predefined_acl: Some(PredefinedObjectAcl::PublicRead),
        ..Default::default()
    };

    // 使用 Multipart 上傳類型
    let upload_type = UploadType::Multipart(Box::new(object));

    // 上傳文件
    let _uploaded = client
        .upload_object(&upload_request, data, &upload_type)
        .await?;

    // 生成公開 URL
    let public_url = format!(
        "https://firebasestorage.googleapis.com/v0/b/{}/o/{}?alt=media&token={}",
        bucket_name,
        urlencoding::encode(filename),
        download_token
    );

    Ok(public_url)
}
