#[macro_use]
extern crate diesel;

use actix_web::http::Method;
use actix_web::App;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenv::dotenv;
use std::env;

mod handlers;
mod schema;
mod model;
mod db;

// アプリケーションで持ち回る状態
#[derive(Clone)]
pub struct Server {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Server {
    pub fn new() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set");
        let manager: ConnectionManager<PgConnection> = ConnectionManager::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        Server { pool }
    }
}

pub fn app(server: Server) -> App<Server> {
    use crate::handlers::*;

    let app: App<Server> = App::with_state(server)
        .route("/csv" , Method::POST, handle_post_csv)  // ログをCSVファイルで受け取っとDBに保存する
        .route("/logs", Method::POST, handle_post_logs) // ログをJSON形式で受け取っとDBに保存する
        .route("/csv" , Method::GET , handle_get_csv)   // DBにあるログをCSVファイルとして返す。(from=timestamp, until=timestampを受け付ける)
        .route("/logs", Method::GET , handle_get_logs); // DBにあるログをJSON形式で返す。(from=timestamp, until=timestampを受け付ける)
    app
}

fn main() {
    env_logger::init();

    let server = Server::new();
    actix_web::server::new(move || app(server.clone()))
        .bind("localhost:3080")
        .expect("Can not bind to port 3000")
        .run();
}
