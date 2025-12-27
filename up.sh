#!/bin/bash

# code . && docker-compose up -d && cargo watch -x run

code .

# 啟動所有服務
docker-compose up -d

# 顯示 migration 日誌直到完成
echo "🔄 Running database migrations..."
docker-compose logs -f sqlx-cli &
LOGS_PID=$!

# 等待 sqlx-cli 容器完成
docker wait sqlx-cli

# 停止日誌追蹤
kill $LOGS_PID 2>/dev/null

# 檢查 migration 結果
EXIT_CODE=$(docker inspect sqlx-cli --format='{{.State.ExitCode}}')

if [ "$EXIT_CODE" -eq 0 ]; then
    echo "✅ Migration completed successfully"
    echo ""
    echo "🚀 Starting application..."
    cargo watch -x run
else
    echo "❌ Migration failed with exit code $EXIT_CODE"
    docker-compose down
    exit 1
fi
