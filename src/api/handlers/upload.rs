// src/api/handlers/upload.rs
use axum::{
    extract::{Multipart, State},
    response::IntoResponse,
};
use google_cloud_storage::{
    client::{Client, ClientConfig, google_cloud_auth::credentials::CredentialsFile},
    http::{
        object_access_controls::PredefinedObjectAcl,
        objects::{
            Object,
            upload::{UploadObjectRequest, UploadType},
        },
    },
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Deserialize)]
struct FirebaseCredentials {
    #[serde(rename = "type")]
    cred_type: String,
    project_id: String,
    private_key_id: String,
    private_key: String,
    client_email: String,
    client_id: String,
    auth_uri: String,
    token_uri: String,
    auth_provider_x509_cert_url: String,
    client_x509_cert_url: String,
}

/// 處理圖片上傳至使用者自己的 Firebase Storage 的 API Handler
///
/// 此函式會解析 multipart 表單，包含：
/// 1. 圖片檔案
/// 2. 使用者的 Firebase credentials.json
/// 3. (可選) Firebase Storage bucket 名稱
///
/// ### 流程：
/// 1. 解析 Multipart 欄位（`image`/`file`, `credentials`, `bucket`）
/// 2. 驗證檔案是否存在
/// 3. 驗證檔案大小（上限 10MB）
/// 4. 驗證 MIME 類型（必須為 `image/*`）
/// 5. 解析並驗證 Firebase credentials
/// 6. 生成 UUID 唯一檔名並上傳到使用者的 Firebase
///
/// ### 參數：
/// * `state`: 全域應用程式狀態
/// * `multipart`: Axum 提供的 Multipart 解析器
pub async fn upload_image(
    State(_state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // --- 1. 從 multipart 中提取資料 ---
    let mut file_data: Option<Vec<u8>> = None;
    let mut original_filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut credentials_json: Option<String> = None;
    let mut bucket_name: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::bad_request(format!("無法解析表單欄位: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "image" | "file" => {
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
            "credentials" => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("無法讀取 credentials: {}", e)))?;

                credentials_json = Some(String::from_utf8(bytes.to_vec()).map_err(|e| {
                    AppError::bad_request(format!("credentials 非有效 UTF-8: {}", e))
                })?);
            }
            "bucket" => {
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::bad_request(format!("無法讀取 bucket 名稱: {}", e)))?;

                bucket_name = Some(String::from_utf8(bytes.to_vec()).map_err(|e| {
                    AppError::bad_request(format!("bucket 名稱非有效 UTF-8: {}", e))
                })?);
            }
            _ => {} // 忽略其他欄位
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

    // --- 4. 驗證 Firebase credentials ---
    let creds_json =
        credentials_json.ok_or_else(|| AppError::bad_request("未提供 Firebase credentials"))?;

    let credentials: FirebaseCredentials = serde_json::from_str(&creds_json)
        .map_err(|e| AppError::bad_request(format!("無效的 Firebase credentials 格式: {}", e)))?;

    // 從 credentials 中取得 project_id 作為預設 bucket 名稱
    let bucket = bucket_name.unwrap_or_else(|| format!("{}.appspot.com", credentials.project_id));

    // --- 5. 生成唯一檔名並執行上傳 ---
    let extension = original_filename
        .as_ref()
        .and_then(|name| name.split('.').last())
        .unwrap_or("jpg");

    let unique_filename = format!("axum-app-uploads/{}.{}", Uuid::new_v4(), extension);
    let file_size = data.len() as u64;

    // 呼叫 Firebase 上傳邏輯
    let url = upload_to_user_firebase(&bucket, &unique_filename, data, &mime_type, &creds_json)
        .await
        .map_err(|e| AppError::internal_error(format!("Firebase 上傳失敗: {}", e)))?;

    // 使用統一的 success 響應
    Ok(response::success(UploadResponse {
        url,
        filename: unique_filename,
        size: file_size,
    }))
}

/// 上傳文件到使用者的 Firebase Storage
async fn upload_to_user_firebase(
    bucket_name: &str,
    filename: &str,
    data: Vec<u8>,
    content_type: &str,
    credentials_json: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    // 建立 Firebase Storage 客戶端
    let config = ClientConfig::default()
        .with_credentials(CredentialsFile::new_from_str(credentials_json).await?)
        .await?;
    let client = Client::new(config);

    let download_token = Uuid::new_v4().to_string();

    let mut metadata = std::collections::HashMap::new();
    metadata.insert(
        "firebaseStorageDownloadTokens".to_string(),
        download_token.clone(),
    );

    let object = Object {
        name: filename.to_string(),
        bucket: bucket_name.to_string(),
        content_type: Some(content_type.to_string()),
        cache_control: Some("public, max-age=31536000".to_string()),
        metadata: Some(metadata),
        ..Default::default()
    };

    let upload_request = UploadObjectRequest {
        bucket: bucket_name.to_string(),
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
