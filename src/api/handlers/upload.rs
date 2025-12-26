// src/api/handlers/upload.rs
use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
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

use crate::api::response;
use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct UploadResponse {
    pub url: String,
    pub filename: String,
    pub size: u64,
}

/// 處理圖片上傳至 Firebase Storage 的 API Handler
///
/// 此函式會解析 multipart 表單，驗證檔案大小與格式，
/// 並將合規的圖片上傳至雲端儲存空間。
///
/// ### 流程：
/// 1. 解析 Multipart 欄位（尋找 `image` 或 `file` 欄位）
/// 2. 驗證檔案是否存在
/// 3. 驗證檔案大小（上限 10MB）
/// 4. 驗證 MIME 類型（必須為 `image/*`）
/// 5. 生成 UUID 唯一檔名並執行上傳
///
/// ### 參數：
/// * `state`: 全域應用程式狀態，內含 Firebase 客戶端或相關配置
/// * `multipart`: Axum 提供的 Multipart 解析器
pub async fn upload_image(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // --- 1. 從 multipart 中提取文件資料 ---
    let mut file_data: Option<Vec<u8>> = None;
    let mut original_filename: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("無法解析表單欄位: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        // 支援 image 或 file 作為 key
        if name == "image" || name == "file" {
            original_filename = field.file_name().map(|s| s.to_string());
            content_type = field.content_type().map(|s| s.to_string());

            file_data = Some(
                field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("無法讀取檔案數據: {}", e)))?
                    .to_vec(),
            );
        }
    }

    // --- 2. 檔案基本驗證 ---
    let data = file_data.ok_or_else(|| AppError::bad_request("未提供檔案"))?;

    // 驗證檔案大小 (限制 10MB)
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    if data.len() > MAX_FILE_SIZE {
        return Err(AppError::payload_too_large("檔案大小超過 10MB 限制"));
    }

    // --- 3. 媒體類型 (MIME) 驗證 ---
    let mime_type = content_type.clone().unwrap_or_else(|| {
        mime_guess::from_path(original_filename.as_ref().unwrap_or(&String::new()))
            .first_or_octet_stream()
            .to_string()
    });

    if !mime_type.starts_with("image/") {
        return Err(AppError::bad_request("只允許上傳圖片檔案"));
    }

    // --- 4. 生成唯一檔名並執行上傳 ---
    let extension = original_filename
        .as_ref()
        .and_then(|name| name.split('.').last())
        .unwrap_or("jpg");

    let unique_filename = format!("axum-app-uploads/{}.{}", Uuid::new_v4(), extension);
    let file_size = data.len() as u64;

    // 呼叫 Firebase 上傳邏輯
    let url = upload_to_firebase(&state, &unique_filename, data, &mime_type)
        .await
        .map_err(|e| AppError::internal_error(format!("Firebase 上傳失敗: {}", e)))?;

    // 使用統一的 success 響應
    Ok(response::success(UploadResponse {
        url,
        filename: unique_filename,
        size: file_size,
    }))
}

/// 上傳文件到 Firebase Storage
async fn upload_to_firebase(
    _state: &AppState,
    filename: &str,
    data: Vec<u8>,
    content_type: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let bucket_name =
        std::env::var("FIREBASE_STORAGE_BUCKET").expect("FIREBASE_STORAGE_BUCKET must be set");

    let config = ClientConfig::default().with_auth().await?;
    let client = Client::new(config);

    let download_token = Uuid::new_v4().to_string();

    let mut metadata = std::collections::HashMap::new();
    metadata.insert(
        "firebaseStorageDownloadTokens".to_string(),
        download_token.clone(),
    );

    let object = Object {
        name: filename.to_string(),
        bucket: bucket_name.clone(),
        content_type: Some(content_type.to_string()),
        cache_control: Some("public, max-age=31536000".to_string()),
        metadata: Some(metadata),
        ..Default::default()
    };

    let upload_request = UploadObjectRequest {
        bucket: bucket_name.clone(),
        predefined_acl: Some(PredefinedObjectAcl::PublicRead),
        ..Default::default()
    };

    let upload_type = UploadType::Multipart(Box::new(object));

    let _uploaded = client
        .upload_object(&upload_request, data, &upload_type)
        .await?;

    let public_url = format!(
        "https://firebasestorage.googleapis.com/v0/b/{}/o/{}?alt=media&token={}",
        bucket_name,
        urlencoding::encode(filename),
        download_token
    );

    Ok(public_url)
}
