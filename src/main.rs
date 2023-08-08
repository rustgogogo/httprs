use std::{collections::HashMap, str::FromStr};
use std::process::exit;

use clap::{Parser, Subcommand};
use colored::*;
use mime::Mime;
use reqwest::{Client, header, Response};

#[derive(Parser, Debug)]
struct CliOpts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post),
}

#[derive(Parser, Debug)]
struct Get {
    url: String,
}

#[derive(Parser, Debug)]
struct Post {
    url: String,
    body: Vec<KvPair>,
}

#[derive(Debug, PartialEq, Clone)]
struct KvPair {
    k: String,
    v: String,
}

impl FromStr for KvPair {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = format!("Invalid Key-Value pair: {}", s);
        Ok(Self {
            k: split.next().ok_or(err.clone())?.to_string(),
            v: split.next().ok_or(err)?.to_string(),
        })
    }
}

async fn get(client: Client, args: &Get) {
    let resp = client.get(&args.url).send().await.unwrap();
    print_resp(resp).await
}

async fn post(client: Client, args: &Post) {
    let mut body = HashMap::new();
    for kv in args.body.iter() {
        body.insert(&kv.k, &kv.v);
    }
    let resp = client.post(&args.url).json(&body).send().await.unwrap();
    print_resp(resp).await
}

fn print_status(resp: &Response) {
    let status = format!("{:?}", resp.status()).blue();
    println!("Status: {} \n", status);
}

fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {}", name.to_string().green(), value.to_str().unwrap());
    }
    println!();
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan());
        }
        _ => println!("{}", body),
    }
}

async fn print_resp(resp: Response) {
    print_status(&resp);
    print_headers(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await.unwrap();
    print_body(mime, &body);
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers().get(header::CONTENT_TYPE)
        .map(|v| v.to_str()
            .unwrap()
            .parse::<Mime>().unwrap())
}

#[tokio::main]
async fn main() {
    let parse_result = CliOpts::try_parse();
    if parse_result.is_err() {
        let err = parse_result.unwrap_err();

        println!("{:?}", err);
        println!("{}", "参数不能为空🤔".yellow());
        exit(0);
    }

    let args = parse_result.unwrap();

    let client = Client::new();
    println!("{:?}", args.subcmd);


    match args.subcmd {
        SubCommand::Get(ref args) => get(client, args).await,
        SubCommand::Post(ref args) => post(client, args).await,
    }
}
