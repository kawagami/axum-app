pub mod health;

// 重新導出常用處理函數，方便引入
pub use health::{health_fail, health_ok};
