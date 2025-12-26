pub mod health;
mod upload;

// 重新導出常用處理函數，方便引入
pub use health::{get_stock_day_all, handler_404, health_fail, health_ok};
pub use upload::upload_image;
