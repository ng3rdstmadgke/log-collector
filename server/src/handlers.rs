use actix_web::{HttpResponse, State, Json, Query};
use log::debug;
use failure::Error;
use crate::Server;
use crate::db;
use actix_web::FutureResponse;
use actix_web_multipart_file::{Multiparts, FormData};
use diesel::pg::PgConnection;
use futures::prelude::*;
use itertools::Itertools;
use std::io::{BufReader, Read};


// POST / csvのハンドラ
// ログをCSVファイルで受け取っとDBに保存する
pub fn handle_post_csv(
    server: State<Server>,
    multiparts: Multiparts, // actix_web_multipart_fileを使ってマルチパートのリクエストをファイルに保存する
) -> FutureResponse<HttpResponse> {
    // multipartsはStreamになっているのでそのままメソッドをつなげる
    // Streamは非同期版のイテレータのような存在
    let fut = multiparts
        .from_err()
        .filter(|field| field.content_type == "text/csv") // text/csvでなければスキップ
        .filter_map(|field| match field.form_data {       // ファイルでなければスキップ
            FormData::File { file, .. } => Some(file),
            FormData::Data { .. }       => None,
        })
        .and_then(move |file| load_file(&*server.pool.get()?, file)) // 1ファイルずつ処理
        .fold(0, |acc, x| Ok::<_, Error>(acc + x))                   // usizeのStream -> それらの和
        .map(|sum| HttpResponse::Ok().json(api::csv::post::Response(sum)))
        .from_err();

    Box::new(fut)
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
    use chrono::{DateTime, Utc};

    let conn = server.pool.get()?;
    let logs = db::logs(&conn, range.from, range.until)?;
    let v = Vec::new();
    let mut w = csv::Writer::from_writer(v);

    for log in logs.into_iter()
        .map(|log| api::Log {
            user_agent: log.user_agent,
            response_time: log.response_time,
            timestamp: DateTime::from_utc(log.timestamp, Utc),
        }) {
            w.serialize(log)?
    }

    let csv = w.into_inner()?;

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

fn load_file(conn: &PgConnection, file: impl Read) -> Result<usize, Error> {
    use crate::model::NewLog;

    let mut ret = 0;

    // csvファイルが渡されるcsv::Reader を用いて api::Log にデコードしていく
    let in_csv = BufReader::new(file);
    let in_log = csv::Reader::from_reader(in_csv).into_deserialize::<::api::Log>();
    // Iteratools の chunks を用いて1000件ずつ処理する
    for logs in &in_log.chunks(1000) {
        let logs = logs
            .filter_map(Result::ok)
            .map(|log| NewLog {
                user_agent: log.user_agent,
                response_time: log.response_time,
                timestamp: log.timestamp.naive_utc(),
            })
            .collect_vec();
        let inserted = db::insert_logs(conn, &logs)?;
        ret += inserted.len();
    }
    Ok(ret)
}
