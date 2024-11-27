use core::str;

use alloc::vec;
use alloc::{format, string::String};
use noli::net::{lookup_host, SocketAddr, TcpStream};
use saba_core::{error::Error, http::HttpResponse};

extern crate alloc;

pub struct HttpClient {}

impl HttpClient {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get(&self, host: String, port: u16, path: String) -> Result<HttpResponse, Error> {
        let ips = match lookup_host(&host) {
            Ok(ips) => ips,
            Err(e) => {
                return Err(Error::Network(format!(
                    "Failed to find IP addresses: {e:#?}"
                )));
            }
        };

        if ips.len() < 1 {
            return Err(Error::Network("Failed to find IP addresses".into()));
        }

        let socket_addr: SocketAddr = (ips[0], port).into();

        let Ok(mut stream) = TcpStream::connect(socket_addr) else {
            return Err(Error::Network("Failed to connect to TCP stream".into()));
        };

        let mut request = String::from("GET /");
        request.push_str(&path);
        request.push_str(" HTTP/1.1\n");

        // ヘッダの追加
        request.push_str("Host: ");
        request.push_str(&host);
        request.push('\n');
        request.push_str("Accept: text/html\n");
        request.push_str("Connection: close\n");
        request.push('\n');

        let Ok(_bytes_written) = stream.write(request.as_bytes()) else {
            return Err(Error::Network(
                "Failed to send a request to TCP stream".into(),
            ));
        };

        let mut received = vec![];
        loop {
            let mut buf = [0; 4096];
            let Ok(bytes_read) = stream.read(&mut buf) else {
                return Err(Error::Network(
                    "Failed to receive a request from TCP stream".into(),
                ));
            };
            if bytes_read == 0 {
                break;
            }
            received.extend_from_slice(&buf[..bytes_read]);
        }

        match str::from_utf8(&received) {
            Ok(response) => HttpResponse::new(response.into()),
            Err(e) => Err(Error::Network(format!("Invalid received response: {e}"))),
        }
    }
}
