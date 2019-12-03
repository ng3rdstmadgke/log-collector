use actix_web::{HttpResponse, State, Json, Query};
use log::debug;
use failure::Error;
use crate::Server;
use crate::db;


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
    use chrono::Utc;
    use crate::model::NewLog;

    let log = NewLog {
        user_agent: log.user_agent.clone(),
        response_time: log.response_time,
        timestamp: log.timestamp.unwrap_or_else(|| Utc::now()).naive_utc(),
    };
    let conn = server.pool.get()?;
    db::insert_log(&conn, &log)?;
    debug!("received log: {:?}", log);
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
    use chrono::{DateTime, Utc};

    let conn = server.pool.get()?;
    let logs = db::logs(&conn, range.from, range.until)?;
    let logs = logs
        .into_iter()
        .map(|log| api::Log {
            user_agent: log.user_agent,
            response_time: log.response_time,
            timestamp: DateTime::from_utc(log.timestamp, Utc),
        })
        .collect();
    Ok(HttpResponse::Ok().json(api::logs::get::Response(logs)))
}
