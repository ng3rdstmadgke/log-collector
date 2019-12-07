use clap::{Arg, App, AppSettings, SubCommand, _clap_count_exprs, arg_enum};
use reqwest::Client;
use std::io;

fn main() {
    let opts = App::new(env!("CARGO_PKG_NAME"))
        // ここまでテンプレ
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        // サブコマンドが必要だという設定
        .setting(AppSettings::SubcommandRequiredElseHelp)
        // -s URL | --server URL のオプションを受け付ける
        .arg(
            Arg::with_name("SERVER")
                .short("s")
                .long("server")
                .value_name("URL")
                .help("server url")
                .takes_value(true),
        )
        // サブコマンドとしてpostを受け付ける
        .subcommand(
            SubCommand::with_name("post")
                .about("post logs, taking input from stdin")
        )
        // サブコマンドとしてgetを受け付ける
        .subcommand(
            SubCommand::with_name("get")
                .about("get logs")
                .arg(
                    Arg::with_name("FORMAT")
                        .short("f")
                        .long("format")
                        .help("log format")
                        .takes_value(true)
                        .possible_values(&Format::variants()) // "csv", "json" のみ受け付ける
                        .case_insensitive(true),
                )
        );
    let matches = opts.get_matches();
    let server: String = matches
        .value_of("SERVER")
        .unwrap_or("localhost:3080")
        .into();
    let client = Client::new();
    let api_client = ApiClient { server, client };

    match matches.subcommand() {
        ("get" , sub_match) => {
            let format = sub_match
                .and_then(|m| m.value_of("FORMAT"))
                .map(|m| m.parse().unwrap()) // Format型にパース
                .unwrap();
            match format {
                Format::Csv  => do_get_csv(&api_client),
                Format::Json => do_get_json(&api_client),
            }
        }
        ("post", _) => do_post_csv(&api_client),
        _ => unreachable!(),
    }
}

arg_enum! {
    #[derive(Debug)]
    enum Format {
        Csv,
        Json,
    }
}

fn do_post_csv(api_client: &ApiClient) {
    let reader = csv::Reader::from_reader(io::stdin());
    for log in reader.into_deserialize::<api::logs::post::Request>() {
        let log = match log {
            Ok(log) => log,
            Err(e)  => {
                eprintln!("[WARN] failed to parse a line, skipping: {}", e);
                continue;
            }
        };
        api_client.post_logs(&log).expect("api request failed");
    }
}

fn do_get_json(api_client: &ApiClient) {
    let res = api_client.get_logs().expect("api request failed");
    let json_str = serde_json::to_string(&res).unwrap();
    println!("{}", json_str);
}

fn do_get_csv(api_client: &ApiClient) {
    let res = api_client.get_logs().expect("api request failed");
    let mut w = csv::Writer::from_writer(io::stdout());
    for log in res.0 {
        w.serialize(log).unwrap();
    }
}

struct ApiClient {
    server: String,
    client: Client,
}

impl ApiClient {
    fn post_logs(&self, req: &api::logs::post::Request) -> reqwest::Result<()> {
        self.client
            .post(&format!("http://{}/logs", &self.server))
            .json(req)
            .send()
            .map(|_| ())
    }

    fn get_logs(&self) -> reqwest::Result<api::logs::get::Response> {
        self.client
            .get(&format!("http://{}/logs", &self.server))
            .send()?
            .json()
    }
}
