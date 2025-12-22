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
///
/// ### 回傳值：
/// * `Ok(Json<UploadResponse>)`: 上傳成功，回傳圖片 URL 與資訊
/// * `Err((StatusCode, Json<ErrorResponse>))`: 上傳失敗，回傳對應的錯誤狀態碼與訊息
pub async fn upload_image(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, (StatusCode, Json<ErrorResponse>)> {
    // --- 1. 從 multipart 中提取文件資料 ---
    let mut file_data: Option<Vec<u8>> = None;
    let mut original_filename: Option<String> = None;
    let mut content_type: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("無法解析表單欄位: {}", e),
            }),
        )
    })? {
        let name = field.name().unwrap_or("").to_string();

        // 支援 image 或 file 作為 key
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
                                error: format!("無法讀取檔案數據: {}", e),
                            }),
                        )
                    })?
                    .to_vec(),
            );
        }
    }

    // --- 2. 檔案基本驗證 ---
    // 檢查是否有上傳檔案
    let data = file_data.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "未提供檔案".to_string(),
            }),
        )
    })?;

    // 驗證檔案大小 (限制 10MB)
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    if data.len() > MAX_FILE_SIZE {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(ErrorResponse {
                error: "檔案大小超過 10MB 限制".to_string(),
            }),
        ));
    }

    // --- 3. 媒體類型 (MIME) 驗證 ---
    let mime_type = content_type.clone().unwrap_or_else(|| {
        mime_guess::from_path(original_filename.as_ref().unwrap_or(&String::new()))
            .first_or_octet_stream()
            .to_string()
    });

    if !mime_type.starts_with("image/") {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "只允許上傳圖片檔案".to_string(),
            }),
        ));
    }

    // --- 4. 生成唯一檔名並執行上傳 ---
    let extension = original_filename
        .as_ref()
        .and_then(|name| name.split('.').last())
        .unwrap_or("jpg");

    // 格式: axum-app-uploads/{uuid}.{ext}
    let unique_filename = format!("axum-app-uploads/{}.{}", Uuid::new_v4(), extension);
    let file_size = data.len() as u64;

    // 呼叫 Firebase 上傳邏輯
    let url = upload_to_firebase(&state, &unique_filename, data, &mime_type)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Firebase 上傳失敗: {}", e),
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
