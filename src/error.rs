// src/error.rs

use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::fmt;

use crate::api::response;

/// 應用程序錯誤類型
pub struct AppError {
    pub status_code: StatusCode,
    pub message: String,
    /// 保存底層錯誤以便調試
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl AppError {
    pub fn new(status_code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status_code,
            message: message.into(),
            source: None,
        }
    }

    /// 帶源錯誤的構造函數
    pub fn with_source(
        status_code: StatusCode,
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            status_code,
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn _unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self::new(StatusCode::PAYLOAD_TOO_LARGE, message)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.status_code, self.message)?;
        if let Some(source) = &self.source {
            write!(f, " (caused by: {})", source)?;
        }
        Ok(())
    }
}

impl fmt::Debug for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppError")
            .field("status_code", &self.status_code)
            .field("message", &self.message)
            .field("source", &self.source)
            .finish()
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn std::error::Error + 'static))
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let message = self.message.clone();

        // 記錄完整錯誤鏈
        if let Some(source) = &self.source {
            tracing::error!(
                status = %self.status_code,
                message = %message,
                source = %source,
                "API Error"
            );
        } else {
            tracing::error!(
                status = %self.status_code,
                message = %message,
                "API Error"
            );
        }

        response::error(self.status_code, message).into_response()
    }
}

// 從各種錯誤類型自動轉換
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        Self::with_source(StatusCode::INTERNAL_SERVER_ERROR, "資料庫錯誤", err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        tracing::error!("HTTP request error: {:?}", err);
        Self::with_source(StatusCode::INTERNAL_SERVER_ERROR, "HTTP 請求失敗", err)
    }
}

impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        Self::with_source(StatusCode::BAD_REQUEST, "日期格式錯誤", err)
    }
}

// // 支持從 eyre::Report 轉換（如果需要）
// #[cfg(feature = "eyre")]
// impl From<eyre::Report> for AppError {
//     fn from(err: eyre::Report) -> Self {
//         tracing::error!("Eyre error: {:?}", err);
//         Self::new(
//             StatusCode::INTERNAL_SERVER_ERROR,
//             format!("內部錯誤: {}", err),
//         )
//     }
// }
