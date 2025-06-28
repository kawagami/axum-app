use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::fmt;

use crate::api::response;

/// 應用程序錯誤類型
pub struct AppError {
    pub status_code: StatusCode,
    pub message: String,
}

impl AppError {
    pub fn new(status_code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status_code,
            message: message.into(),
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.status_code, self.message)
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AppError {{ status_code: {}, message: {} }}",
            self.status_code, self.message
        )
    }
}

impl std::error::Error for AppError {}

// 實現 IntoResponse，這樣可以直接從 Route Handler 返回 Result<T, AppError>
impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let message = self.message.clone();
        tracing::error!("API Error: {}", &self);

        // 使用之前定義的 error 函數創建錯誤響應
        response::error(self.status_code, message).into_response()
    }
}

// 從 sqlx 錯誤類型自動轉換為 AppError
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {}", err);
        Self::internal_error(format!("Database error: {}", err))
    }
}

// 從 reqwest 錯誤類型自動轉換為 AppError
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        tracing::error!("HTTP request error: {}", err);
        Self::internal_error(format!("HTTP request error: {}", err))
    }
}

// 從 reqwest 錯誤類型自動轉換為 AppError
impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        tracing::error!("chrono failed to parse: {}", err);
        Self::internal_error(format!("chrono failed to parse: {}", err))
    }
}
