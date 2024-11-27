#![no_std]
#![cfg_attr(not(target_os = "linux"), no_main)]

use net_wasabi::http::HttpClient;
use noli::prelude::*;

entry_point!(main);

fn main() -> u64 {
    let client = HttpClient::new();
    match client.get("host.test".into(), 8000, "/test.html".into()) {
        Ok(res) => print!("response:\n{res:#?}"),
        Err(e) => print!("error:\n{e:#?}"),
    }
    0
}
