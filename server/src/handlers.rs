use actix_web::{HttpResponse, State, Json, Query};
use log::debug;
use failure::Error;
use crate::Server;


// POST / csvのハンドラ
// ログをCSVファイルで受け取っとDBに保存する
pub fn handle_post_csv(server: State<Server>) -> Result<HttpResponse, Error> {
    // POST されたファイルはActix Webでは簡単には扱えないのでここではまだコードなし
    // レスポンスはDefaultでダミーデータを生成
    let logs = Default::default();
    Ok(HttpResponse::Ok().json(api::csv::post::Response(logs)))
}

// POST / logsのハンドラ
// ログをJSON形式で受け取っとDBに保存する
pub fn handle_post_logs(
    server: State<Server>,
    log: Json<api::logs::post::Request>, // POSTのボディはJson<T>を引数に書くと自動的にデシリアライズされて渡される
) -> Result<HttpResponse, Error> {
    // Json<T>はTへのDDerefを実装しているので内部ではほぼそのままTの値として扱える
    debug!("{:?}", log);
    // レスポンスはAccepted
    Ok(HttpResponse::Accepted().finish())
}

// GET / csvのハンドラ
// DBにあるログをCSVファイルとして返す。(from=timestamp, until=timestampを受け付ける)
pub fn handle_get_csv(
    server: State<Server>,
    range: Query<api::csv::get::Query>,
) -> Result<HttpResponse, Error> {
    debug!("{:?}", range);

    // CSVファイルはバイナリデータにして返す
    let csv: Vec<u8> = vec![];
    Ok(HttpResponse::Ok().header("Content-Type", "text/csv").body(csv))
}

// POST / logsのハンドラ
// DBにあるログをJSON形式で返す。(from=timestamp, until=timestampを受け付ける)
pub fn handle_get_logs(
    server: State<Server>,
    range: Query<api::logs::get::Query>,
) -> Result<HttpResponse, Error> {
    debug!("{:?}", range);
    let logs = Default::default();

    Ok(HttpResponse::Ok().json(api::logs::get::Response(logs)))
}
