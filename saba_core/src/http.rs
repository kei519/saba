use alloc::vec;
use alloc::{format, string::String, vec::Vec};

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct HttpResponse {
    version: String,
    status_code: u32,
    reason: String,
    headers: Vec<Header>,
    body: String,
}

impl HttpResponse {
    pub fn new(raw_response: String) -> Result<Self, Error> {
        let preprocessed_response = raw_response.trim_start().replace("\n\r", "\n");

        let (status_line, remaining) = match preprocessed_response.split_once('\n') {
            Some(splited) => splited,
            None => {
                return Err(Error::Network(format!(
                    "invalid http response: {preprocessed_response}"
                )));
            }
        };

        let (headers, body) = match remaining.split_once("\n\n") {
            Some((h, b)) => {
                let mut headers = vec![];
                for header in h.split('\n') {
                    let splitted_header: Vec<_> = header.splitn(2, ':').collect();
                    headers.push(Header::new(
                        splitted_header[0].trim().into(),
                        splitted_header[1].trim().into(),
                    ));
                }
                (headers, b)
            }
            None => (vec![], remaining),
        };

        let statuses: Vec<_> = status_line.split(' ').collect();
        if statuses.len() != 3 {
            return Err(Error::Network(format!(
                "invalid http response: {preprocessed_response}"
            )));
        }

        Ok(Self {
            version: statuses[0].into(),
            status_code: statuses[1].parse().unwrap_or(404),
            reason: statuses[2].into(),
            headers,
            body: body.into(),
        })
    }

    pub fn version(&self) -> String {
        self.version.clone()
    }

    pub fn status_code(&self) -> u32 {
        self.status_code
    }

    pub fn reason(&self) -> String {
        self.reason.clone()
    }

    pub fn headers(&self) -> Vec<Header> {
        self.headers.clone()
    }

    pub fn body(&self) -> String {
        self.body.clone()
    }

    pub fn header_value(&self, name: &str) -> Result<String, String> {
        if let Some(value) = self
            .headers
            .iter()
            .find_map(|h| (h.name == name).then_some(h.value.clone()))
        {
            Ok(value)
        } else {
            Err(format!("failed to find {name} in headers"))
        }
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    name: String,
    value: String,
}

impl Header {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_status_line_only() {
        let raw = "HTTP/1.1 200 OK\n\n".into();
        let res = HttpResponse::new(raw).expect("failed to parse http response");
        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), 200);
        assert_eq!(res.reason(), "OK");
    }

    #[test]
    fn test_one_header() {
        let raw = "HTTP/1.1 200 OK\nDate:xx xx xx\n\n".into();
        let res = HttpResponse::new(raw).expect("afiled to parse http response");
        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), 200);
        assert_eq!(res.reason(), "OK");

        assert_eq!(res.header_value("Date"), Ok("xx xx xx".into()))
    }

    #[test]
    fn test_two_headers() {
        let raw = "HTTP/1.1 200 OK\nDate: xx xx xx\nContent-Length: 42\n\n".into();
        let res = HttpResponse::new(raw).expect("failed to parse http response");
        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), 200);
        assert_eq!(res.reason(), "OK");

        assert_eq!(res.header_value("Date"), Ok("xx xx xx".into()));
        assert_eq!(res.header_value("Content-Length"), Ok("42".into()));
    }

    #[test]
    fn test_body() {
        let raw = "HTTP/1.1 200 OK\nDate: xx xx xx\n\nbody message".into();
        let res = HttpResponse::new(raw).expect("failed to parse http response");
        assert_eq!(res.version(), "HTTP/1.1");
        assert_eq!(res.status_code(), 200);
        assert_eq!(res.reason(), "OK");

        assert_eq!(res.header_value("Date"), Ok("xx xx xx".into()));

        assert_eq!(res.body(), "body message".to_string());
    }

    #[test]
    fn test_invalid() {
        let raw = "HTTP/1.1 200 OK".into();
        assert!(HttpResponse::new(raw).is_err());
    }
}
