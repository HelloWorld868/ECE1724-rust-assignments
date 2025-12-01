use std::collections::BTreeMap;
use std::process;
use structopt::StructOpt;
use url::{ParseError, Url};

#[derive(StructOpt, Debug)]
#[structopt(name = "curl")]
struct Opt {
    url: String,

    #[structopt(short = "X", long, default_value = "GET")]
    method: String,

    #[structopt(short = "d", long)]
    data: Option<String>,

    #[structopt(long)]
    json: Option<String>,
}

fn main() {
    let opt = Opt::from_args();

    // Line 1: Requesting URL
    println!("Requesting URL: {}", opt.url);

    // Line 2: Method
    let mut request_method = opt.method.to_uppercase();
    if opt.json.is_some() {
        request_method = "POST".to_string();
    }
    println!("Method: {}", request_method);

    // Handle --json or -d
    let (request_data, request_header) = if let Some(json_data) = opt.json {
        println!("JSON: {}", json_data);
        if let Err(e) = serde_json::from_str::<serde_json::Value>(&json_data) {
            panic!("Invalid JSON: {}", e);
        }
        (Some(json_data), Some("application/json".to_string()))
    } else if let Some(data) = opt.data {
        println!("Data: {}", data);
        (Some(data), Some("application/x-www-form-urlencoded".to_string()))
    } else {
        (None, None)
    };

    // Parse and validate URL
    let parsed_url = match Url::parse(&opt.url) {
        Ok(u) => {
            if !["http", "https"].contains(&u.scheme()) {
                println!("Error: The URL does not have a valid base protocol.");
                process::exit(1);
            }
            u
        }
        Err(ParseError::InvalidIpv4Address) => {
            println!("Error: The URL contains an invalid IPv4 address.");
            process::exit(1);
        }
        Err(ParseError::InvalidIpv6Address) => {
            println!("Error: The URL contains an invalid IPv6 address.");
            process::exit(1);
        }
        Err(ParseError::InvalidPort) => {
            println!("Error: The URL contains an invalid port number.");
            process::exit(1);
        }
        Err(_) => {
            println!("Error: The URL does not have a valid base protocol.");
            process::exit(1);
        }
    };

    // Build request: client
    let client = reqwest::blocking::Client::new();

    // Build request: builder
    let mut request = client.request(
        reqwest::Method::from_bytes(request_method.as_bytes()).unwrap_or(reqwest::Method::GET),  // method
        parsed_url.to_string(),  //url
    );

    // Build request: body
    if let Some(data) = request_data {
        request = request.body(data);
    }

    // Build request: header
    if let Some(ct) = request_header {
        request = request.header(reqwest::header::CONTENT_TYPE, ct);
    }

    // Send request
    let response: reqwest::blocking::Response = match request.send() {
        Ok(resp) => resp,
        Err(e) => {
            if e.is_connect() || e.is_timeout() {
                println!("Error: Unable to connect to the server. Perhaps the network is offline or the server hostname cannot be resolved.");
            } else {
                println!("Error: {}", e);
            }
            process::exit(1);
        }
    };

    // Check status
    if !response.status().is_success() {
        println!("Error: Request failed with status code: {}.", response.status().as_u16());
        process::exit(1);
    }

    // Get body
    let body = match response.text() {
        Ok(b) => b,
        Err(e) => {
            println!("Error: {}", e);
            process::exit(1);
        }
    };

    // Parse and print JSON with sorted keys
    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&body) {
        println!("Response body (JSON with sorted keys):");
        match json_value {
            serde_json::Value::Object(map) => {
                let sorted: BTreeMap<String, serde_json::Value> = map.into_iter().collect();
                println!("{}", serde_json::to_string_pretty(&sorted).unwrap());
            }
            other => {
                println!("{}", serde_json::to_string_pretty(&other).unwrap());
            }
        }
    } else {
        println!("Response body:");
        println!("{}", body);
    }
}