# 使い方

## サーバー起動
```bash
RUST_LOG=server=debug cargo run --bin server
```

## cliによる操作
### ログの登録
```bash
# csv file
cat test.csv | cargo run --bin cli  -- post
```

### ログの出力
```bash
# csv
cargo run --bin cli  -- get --format csv

# json
cargo run --bin cli  -- get --format json
```

## curlによる操作
### ログの登録
```bash
# csv file
curl -F "file=@test.csv;type=text/csv" "http://localhost:3080/csv"

# json
curl -v -H 'Content-Type: application/json' -d '{"user_agent": "Mozilla", "response_time": 200}' localhost:3080/logs
```

### ログの出力
```bash
# csv
curl localhost:3080/csv

# json
curl localhost:3080/logs
```
