use axum::http::StatusCode;
use axum::{Json, response::IntoResponse};
use serde::Serialize;

/// API 回應結構
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiErrorInfo>,
}

/// API 錯誤信息結構
#[derive(Serialize)]
pub struct ApiErrorInfo {
    pub code: u16,
    pub message: String,
}

/// 創建成功回應
pub fn success<T: Serialize>(data: T) -> impl IntoResponse {
    let response = ApiResponse {
        success: true,
        data: Some(data),
        error: None,
    };
    (StatusCode::OK, Json(response))
}

/// 創建錯誤回應
pub fn error(status: StatusCode, message: impl Into<String>) -> impl IntoResponse {
    let response = ApiResponse::<()> {
        success: false,
        data: None,
        error: Some(ApiErrorInfo {
            code: status.as_u16(),
            message: message.into(),
        }),
    };
    (status, Json(response))
}
