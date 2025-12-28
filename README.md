# 重構 axum 專案
* rust 1.92.0

# 目前目標
* [google & discord ... 等等的 oauth](https://gemini.google.com/app/aa8f2c84dd3afb9a?hl=zh-TW)

# 未來預計目標
* 記帳功能

# 已經完成功能
* google firebase 上傳圖片的 API
    * [討論串](https://claude.ai/chat/8af3a13a-6884-42b9-9574-ee1c3de1fbf7)
    * 需要傳入可使用的 credentials
    * 會上傳圖片到該 credentials 的 firebase 空間中
* [增加快取功能](https://claude.ai/chat/72ce6834-48b3-43ef-ad94-cd445291df20)
* migration 功能從主專案分離出來為一個獨立的 service
    * 使用 sh exec-sqlx-cli.sh sqlx XXXXXX 執行 sqlx 對應功能
