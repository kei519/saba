use alloc::{string::String, vec::Vec};

#[derive(Debug, Clone, PartialEq)]
pub struct Url {
    url: String,
    host: String,
    port: String,
    path: String,
    searchpart: String,
}

impl Url {
    pub fn new(url: String) -> Self {
        Self {
            url,
            host: "".into(),
            port: "".into(),
            path: "".into(),
            searchpart: "".into(),
        }
    }

    pub fn parse(&mut self) -> Result<Self, String> {
        if !self.is_http() {
            return Err("Only HTTP scheme is supported.".into());
        }

        self.host = self.extract_host();
        self.port = self.extract_port();
        self.path = self.extract_path();
        self.searchpart = self.extract_searchpart();

        Ok(self.clone())
    }

    pub fn host(&self) -> String {
        self.host.clone()
    }

    pub fn port(&self) -> String {
        self.port.clone()
    }

    pub fn path(&self) -> String {
        self.port.clone()
    }

    pub fn searchpart(&self) -> String {
        self.searchpart.clone()
    }

    fn is_http(&self) -> bool {
        self.url.starts_with("http://")
    }

    fn extract_host(&self) -> String {
        let url_parts: Vec<_> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, '/')
            .collect();

        if let Some(index) = url_parts[0].find(':') {
            url_parts[0][..index].into()
        } else {
            url_parts[0].into()
        }
    }

    fn extract_port(&self) -> String {
        let url_parts: Vec<_> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, '/')
            .collect();

        if let Some(index) = url_parts[0].find(':') {
            url_parts[0][index + 1..].into()
        } else {
            "80".into()
        }
    }

    fn extract_path(&self) -> String {
        let url_parts: Vec<_> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, '/')
            .collect();

        if let Some(path_and_searchpart) = url_parts.get(1) {
            path_and_searchpart.split('?').next().unwrap().into()
        } else {
            "".into()
        }
    }

    fn extract_searchpart(&self) -> String {
        let url_parts: Vec<_> = self
            .url
            .trim_start_matches("http://")
            .splitn(2, '?')
            .collect();

        if let Some(&searchpart) = url_parts.get(1) {
            searchpart.into()
        } else {
            "".into()
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn test_url_host() {
        let url = "http://example.com".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".into(),
            port: "80".into(),
            path: "".into(),
            searchpart: "".into(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }

    #[test]
    fn test_url_host_port() {
        let url = "http://example.com:8888".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".into(),
            port: "8888".into(),
            path: "".into(),
            searchpart: "".into(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }

    #[test]
    fn test_url_host_port_path() {
        let url = "http://example.com:8888/index.html".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".into(),
            port: "8888".into(),
            path: "index.html".into(),
            searchpart: "".into(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }

    #[test]
    fn test_url_host_path() {
        let url = "http://example.com/index.html".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".into(),
            port: "80".into(),
            path: "index.html".into(),
            searchpart: "".into(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }

    #[test]
    fn test_url_host_port_path_searchquery() {
        let url = "http://example.com:8888/index.html?a=123&b= 456".to_string();
        let expected = Ok(Url {
            url: url.clone(),
            host: "example.com".into(),
            port: "8888".into(),
            path: "index.html".into(),
            searchpart: "a=123&b= 456".into(),
        });
        assert_eq!(expected, Url::new(url).parse());
    }

    #[test]
    fn test_no_schme() {
        let url = "example.com".to_string();
        let expected = Err("Only HTTP scheme is supported.".into());
        assert_eq!(expected, Url::new(url).parse());
    }

    #[test]
    fn test_unsupported_scheme() {
        let url = "https://example.com:8888/index.html".to_string();
        let expected = Err("Only HTTP scheme is supported.".into());
        assert_eq!(expected, Url::new(url).parse());
    }
}
