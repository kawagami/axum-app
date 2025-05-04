use color_eyre::eyre::Report;

/// 自定義錯誤類型
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Internal server error: {0}")]
    Internal(String),
    // #[error("Not found: {0}")]
    // NotFound(String),

    // #[error("Bad request: {0}")]
    // BadRequest(String),

    // #[error("Unauthorized: {0}")]
    // Unauthorized(String),
}

impl AppError {
    // /// 獲取對應的 HTTP 狀態碼
    // pub fn status_code(&self) -> StatusCode {
    //     match self {
    //         Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
    //         Self::NotFound(_) => StatusCode::NOT_FOUND,
    //         Self::BadRequest(_) => StatusCode::BAD_REQUEST,
    //         Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
    //     }
    // }
}

/// 從 color_eyre::Report 轉換為 AppError
impl From<Report> for AppError {
    fn from(err: Report) -> Self {
        tracing::error!("{:?}", err);
        Self::Internal(err.to_string())
    }
}

// /// 便捷函數，用於創建各種常見錯誤
// pub mod errors {
//     use super::*;

//     pub fn internal(message: impl Into<String>) -> AppError {
//         AppError::Internal(message.into())
//     }

//     pub fn not_found(message: impl Into<String>) -> AppError {
//         AppError::NotFound(message.into())
//     }

//     pub fn bad_request(message: impl Into<String>) -> AppError {
//         AppError::BadRequest(message.into())
//     }

//     pub fn unauthorized(message: impl Into<String>) -> AppError {
//         AppError::Unauthorized(message.into())
//     }
// }
