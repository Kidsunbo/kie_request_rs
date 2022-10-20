use std::{collections::HashMap, str::FromStr, time, vec, fmt::format};

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{self, ToSocketAddrs},
};

use url_parse::core::Parser;

use super::Response;
use crate::error;

#[derive(Default)]
pub enum RequestMethod {
    #[default]
    GET,

    #[allow(dead_code)]
    POST,
}

#[derive(Default)]
pub struct Request<'a> {
    url: &'a str,
    consume_time: time::Duration,
    result: Response,
    header: HashMap<String, String>,
    method: RequestMethod,
    body: String,
    header_length: usize,
}

impl<'a> Request<'a> {
    pub fn new(url: &'a str, method: RequestMethod) -> Self {
        Self {
            url: url,
            method: method,
            ..Default::default()
        }
    }

    pub fn add_header<T: Into<String>>(&mut self, key: T, val: T) -> &mut Self {
        let key: String = key.into().to_lowercase();
        let val: String = val.into();
        self.header_length += key.len() + 2 + val.len() + 2;
        self.header.insert(key, val);
        self
    }

    #[allow(dead_code)]
    pub fn remove_header(&mut self, key: String) -> &mut Self {
        match self.header.remove(&key) {
            Some(value) => self.header_length -= key.len() - 2 - value.len() - 2,
            None => (),
        };
        self
    }

    #[allow(dead_code)]
    pub fn set_body(&mut self, value: String) -> &mut Self {
        self.body = value;
        self
    }

    pub async fn get_content<'b, T>(
        &'b mut self,
        _url: T,  //for now, this is useless
    ) -> Result<&'b Response, Box<dyn std::error::Error>>
    where
        T: ToSocketAddrs + AsRef<str>,
    {
        let start_time = time::SystemTime::now();

        let url = match Parser::new(None).parse(self.url) {
            Ok(url) => url,
            Err(_) => return Err(Box::new(error::RequestError::ParseUrlError)),
        };
        let path = match (&url.subdomain, &url.domain, &url.top_level_domain, &url.port) {
            (Some(subdomain), Some(domain), Some(top_level_domain), Some(port)) => format!("{}.{}.{}:{}", subdomain, domain, top_level_domain, port),
            _ => return Err(Box::new(error::RequestError::ParseUrlError)),
        };

        let addr = net::lookup_host(&path)
            .await?
            .filter(|p| p.is_ipv4())
            .take(1)
            .last()
            .expect(format!("failed to find any IP address with {}", path).as_str());
        let sock = net::TcpSocket::new_v4()?;

        let mut sock = sock.connect(addr).await?;

        let mut s_content = String::with_capacity(20 + self.header_length + self.body.len());
        match self.method {
            RequestMethod::GET => s_content.push_str("GET "),
            RequestMethod::POST => s_content.push_str("POST "),
        }

        let mut path = match url.path {
            Some(val) => val.join("/"),
            _ => "/".to_string(),
        };
        if path == "" {
            path.push_str("/");
        }

        s_content.push_str(&path);
        s_content.push_str(" HTTP/1.1\r\n");

        if !self.header.contains_key("content-length") {
            self.add_header("Content-Length".to_string(), self.body.len().to_string());
        }

        self.header
            .iter()
            .for_each(|(k, v)| s_content.push_str(format!("{}: {}\r\n", k, v).as_str()));
        s_content.push_str("\r\n");
        s_content.push_str(&self.body);

        sock.write(s_content.as_bytes()).await?;

        let mut reader = BufReader::new(sock);
        self.parse_response_first_line(&mut reader).await?;
        self.parse_response_header(&mut reader).await?;
        self.parse_response_body(&mut reader).await?;
        if let Ok(time) = start_time.elapsed() {
            self.consume_time = time;
        }

        Ok(&self.result)
    }

    async fn parse_response_first_line(
        &mut self,
        reader: &mut BufReader<tokio::net::TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf = String::new();
        reader.read_line(&mut buf).await?;
        let pieces = buf
            .split_ascii_whitespace()
            .map(|x| x.trim())
            .collect::<Vec<&str>>();
        if pieces.len() < 3 {
            return Err(Box::new(error::RequestError::ParseHeaderError(buf)));
        }
        self.result.set_http_version(pieces[0]);
        self.result.set_status_code(pieces[1]);
        self.result.set_reason(pieces[2]);

        Ok(())
    }

    async fn parse_response_header(
        &mut self,
        reader: &mut BufReader<tokio::net::TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            let mut buf = Vec::<u8>::new();
            if let Ok(num) = reader.read_until(b'\n', &mut buf).await {
                if num < 2 || buf.len() < 2 || buf[0] == b'\r' && buf[1] == b'\n' {
                    break;
                }
            }
            let header = String::from_utf8(buf)?;
            match header.split_once(":") {
                Some((k, v)) => self.result.add_header(
                    String::from_str(k)?.trim(),
                    String::from_str(v)?.trim_start(),
                ),
                None => return Err(Box::new(error::RequestError::ParseHeaderError(header))),
            };
        }
        Ok(())
    }

    async fn parse_response_body(
        &mut self,
        reader: &mut BufReader<tokio::net::TcpStream>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.result.get_content_length() == 0 {
            return Ok(());
        }
        let mut buf = vec![0u8; self.result.get_content_length()];
        reader.read_exact(&mut buf).await?;
        self.result.set_body(String::from_utf8(buf)?);
        Ok(())
    }
}
