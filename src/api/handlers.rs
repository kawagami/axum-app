pub mod health;

// 重新導出常用處理函數，方便引入
pub use health::{get_stock_day_all, health_fail, health_ok};
