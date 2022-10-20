use std::{collections::HashMap, time};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{self, ToSocketAddrs},
};

use url_parse::core::Parser;

use crate::error;

pub enum RequestMethod {
    GET,

    #[allow(dead_code)]
    POST,
}

pub struct Request<'a> {
    pub url: &'a str,
    pub consume_time: time::Duration,
    pub result: String,
    pub header: HashMap<String, String>,
    pub method: RequestMethod,
    pub body: String,
}

impl<'a> Request<'a> {
    pub fn new(url: &'a str, method: RequestMethod) -> Self {
        Self {
            url: url,
            method: method,
            consume_time: Default::default(),
            result: Default::default(),
            header: Default::default(),
            body: Default::default(),
        }
    }

    pub fn add_header<T: Into<String>>(&mut self, key: T, val: T) -> &mut Self {
        self.header.insert(key.into().to_lowercase(), val.into());
        self
    }

    #[allow(dead_code)]
    pub fn add_body(&mut self, value: String) -> &mut Self {
        self.body = value;
        self
    }

    pub async fn get_content<T>(&mut self, url: T) -> Result<String, Box<dyn std::error::Error>>
    where
        T: ToSocketAddrs + AsRef<str>,
    {
        let start_time = time::SystemTime::now();
        let addr = net::lookup_host(&url)
            .await?
            .filter(|p| p.is_ipv4())
            .take(1)
            .last()
            .expect(format!("failed to find any IP address with {}", url.as_ref()).as_str());
        let sock = net::TcpSocket::new_v4()?;
        let mut sock = sock.connect(addr).await?;

        let mut length_of_header = 0;
        self.header
            .iter()
            .for_each(|x| length_of_header += x.0.len() + x.1.len() + 4);
        let mut s_content = String::with_capacity(20 + length_of_header + self.body.len());
        match self.method {
            RequestMethod::GET => s_content.push_str("GET "),
            RequestMethod::POST => s_content.push_str("POST "),
        }

        let mut path = match Parser::new(None).parse(self.url) {
            Ok(url) => match url.path {
                Some(val) => val.join("/"),
                _ => "/".to_string(),
            },
            Err(_) => return Err(Box::new(error::RequestError::ParseUrlError)),
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
        let header = self.get_response_header(&mut reader).await?;
        if let Ok(time) = start_time.elapsed() {
            self.consume_time = time;
        }

        Ok(header)
    }

    async fn get_response_header(
        &self,
        reader: &mut BufReader<tokio::net::TcpStream>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut result = String::new();
        loop {
            let mut buf = Vec::<u8>::new();
            if let Ok(num) = reader.read_until(b'\n', &mut buf).await {
                if num < 2 || buf.len() < 2 || buf[0] == b'\r' && buf[1] == b'\n' {
                    break;
                }
            }
            result.push_str(String::from_utf8(buf)?.as_str())
        }
        Ok(result)
    }
}
