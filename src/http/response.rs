use std::{collections::HashMap, default, hash::Hash};

#[derive(Default, Clone)]
pub enum HTTPVersion {
    #[default]
    UnknwonVersion,
    HTTP1_0(&'static str),
    HTTP1_1(&'static str),
}

#[derive(Default)]
pub struct Response {
    status_code: i16,
    http_version: HTTPVersion,
    reason: String,
    headers: HashMap<String, String>,
    body: String,
    content_length: usize,
}

impl Response {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_http_version(&mut self, val: &str) -> &mut Self {
        self.http_version = match val {
            "HTTP/1.0" => HTTPVersion::HTTP1_0("HTTP/1.0"),
            "HTTP/1.1" => HTTPVersion::HTTP1_1("HTTP/1.1"),
            _ => HTTPVersion::UnknwonVersion,
        };
        self
    }

    pub fn get_http_version<'a>(&'a self) -> &'a HTTPVersion {
        &self.http_version
    }

    pub fn set_content_length(&mut self, val: usize) -> &mut Self {
        self.content_length = val;
        self
    }

    pub fn get_content_length(&self) -> usize {
        self.content_length
    }

    pub fn set_status_code(&mut self, val: &str) -> &mut Self {
        if let Ok(val) = val.trim().parse::<i16>() {
            self.status_code = val;
        }
        self
    }

    pub fn get_status_code(&self) -> i16 {
        self.status_code
    }

    pub fn set_reason(&mut self, val: &str) -> &mut Self {
        self.reason = val.to_string();
        self
    }

    pub fn get_reason<'a>(&'a self) -> &'a String {
        &self.reason
    }

    pub fn add_header<T: Into<String>>(&mut self, key: T, val: T) -> &mut Self {
        let key: String = key.into();
        let val: String = val.into();
        match key.to_lowercase().as_str() {
            "content-length" => self.content_length = val.parse::<usize>().unwrap_or_default(),
            _ => (),
        }
        self.headers.insert(key, val.trim_end().to_string());
        self
    }

    pub fn get_header<'a>(&'a mut self, key: &str) -> Option<&'a String> {
        self.headers.get(key)
    }

    pub fn get_headers<'a>(&'a mut self) -> &'a HashMap<String, String> {
        &self.headers
    }

    pub fn set_body(&mut self, val: String) -> &mut Self {
        self.body = val;
        self
    }

    pub fn get_body<'a>(&'a mut self) -> &'a String {
        &self.body
    }

    pub fn to_string(&self) -> String {
        let mut length = 0usize;
        self.headers
            .iter()
            .for_each(|(x, y)| length += x.len() + y.len() + 4);
        let mut result = String::with_capacity(length + 20 + self.content_length);
        match self.http_version {
            HTTPVersion::HTTP1_0(s) => result.push_str(s),
            HTTPVersion::HTTP1_1(s) => result.push_str(s),
            _ => (),
        };
        result.push_str(" ");
        result.push_str(self.status_code.to_string().as_str());
        result.push_str(" ");
        result.push_str(&self.reason);
        for (k, v) in self.headers.iter() {
            result.push_str(format!("{}: {}\r\n", k, v).as_str());
        }
        result.push_str("\r\n");
        result.push_str(&self.body.as_str());

        result
    }
}
