use std::sync::Arc;

use crate::{
    api::response::{error, success},
    error::AppError,
    state::AppState,
};
use axum::{extract::State, http::StatusCode};
use chrono::NaiveDate;
use color_eyre::eyre::eyre;
use serde::Deserialize;

/// 健康檢查 - OK 路由處理函數
pub async fn health_ok(
    State(state): State<Arc<AppState>>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    // db 查詢
    let rows = sqlx::query("SELECT * FROM users")
        .fetch_all(&state.db)
        .await?;

    // http 請求
    let res = state.http_client.get("https://example.com").send().await?;

    tracing::info!("Users: {}, Http status: {}", rows.len(), res.status());

    Ok(success("ok"))
}

/// 健康檢查 - 故意失敗的路由處理函數
pub async fn health_fail() -> impl axum::response::IntoResponse {
    let err = eyre!("Intentional error");
    tracing::error!("{:?}", err); // 印完整 backtrace + source
    error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

#[derive(Deserialize, Debug)]
struct TwseApiResponse {
    date: String,
    data: Vec<Vec<String>>,
}

/// 取公開資訊觀測站 當日日成交資訊 資料並且整理進資料庫
pub async fn get_stock_day_all(
    State(state): State<Arc<AppState>>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let url = "https://www.twse.com.tw/exchangeReport/STOCK_DAY_ALL";

    let resp: TwseApiResponse = state.http_client.get(url).send().await?.json().await?;

    let trade_date = NaiveDate::parse_from_str(&resp.date, "%Y%m%d")?;

    let parse_i64 = |s: &str| s.replace(",", "").parse::<i64>().ok();
    let parse_f64 = |s: &str| s.replace(",", "").parse::<f64>().ok();

    // 收集欄位資料（每欄一個 Vec）
    let mut trade_dates = Vec::new();
    let mut stock_codes = Vec::new();
    let mut stock_names = Vec::new();
    let mut trade_volumes = Vec::new();
    let mut trade_amounts = Vec::new();
    let mut open_prices = Vec::new();
    let mut high_prices = Vec::new();
    let mut low_prices = Vec::new();
    let mut close_prices = Vec::new();
    let mut price_changes = Vec::new();
    let mut transaction_counts = Vec::new();

    for row in &resp.data {
        if row.len() < 10 {
            continue;
        }

        if let (
            Some(trade_volume),
            Some(trade_amount),
            Some(open_price),
            Some(high_price),
            Some(low_price),
            Some(close_price),
            Some(price_change),
        ) = (
            parse_i64(&row[2]),
            parse_i64(&row[3]),
            parse_f64(&row[4]),
            parse_f64(&row[5]),
            parse_f64(&row[6]),
            parse_f64(&row[7]),
            parse_f64(&row[8]),
        ) {
            let transaction_count = parse_i64(&row[9]).unwrap_or(0) as i32;

            trade_dates.push(trade_date);
            stock_codes.push(row[0].as_str());
            stock_names.push(row[1].as_str());
            trade_volumes.push(trade_volume);
            trade_amounts.push(trade_amount);
            open_prices.push(open_price);
            high_prices.push(high_price);
            low_prices.push(low_price);
            close_prices.push(close_price);
            price_changes.push(price_change);
            transaction_counts.push(transaction_count);
        }
    }

    let query = r#"
        INSERT INTO stock_day_all (
            trade_date, stock_code, stock_name,
            trade_volume, trade_amount, open_price,
            high_price, low_price, close_price,
            price_change, transaction_count
        )
        SELECT * FROM UNNEST(
            $1::date[], $2::text[], $3::text[],
            $4::bigint[], $5::bigint[], $6::double precision[],
            $7::double precision[], $8::double precision[], $9::double precision[],
            $10::double precision[], $11::int[]
        )
        ON CONFLICT (trade_date, stock_code) DO NOTHING;
    "#;

    sqlx::query(query)
        .bind(&trade_dates)
        .bind(&stock_codes)
        .bind(&stock_names)
        .bind(&trade_volumes)
        .bind(&trade_amounts)
        .bind(&open_prices)
        .bind(&high_prices)
        .bind(&low_prices)
        .bind(&close_prices)
        .bind(&price_changes)
        .bind(&transaction_counts)
        .execute(&state.db)
        .await?;

    Ok(success("成功"))
}
