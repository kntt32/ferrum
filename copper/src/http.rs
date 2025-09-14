use crate::url::Url;
use std::collections::HashMap;
use std::str::FromStr;

pub type HttpResult<T> = Result<T, &'static str>;

#[derive(Clone, Debug)]
pub struct HttpRequest {
    version: String,
    method: String,
    url: String,
    headers: HashMap<String, String>,
    content: Option<Vec<u8>>,
}

impl HttpRequest {
    pub fn new(method: &str, url: Url<'_>) -> Self {
        Self {
            version: String::from("HTTP/1.1"),
            method: method.into(),
            headers: [
                ("Connection".into(), "close".into()),
                ("Host".into(), url.host().into()),
            ]
            .into(),
            url: url.as_str().into(),
            content: None,
        }
    }

    pub fn header(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn push_header(&mut self, name: String, value: String) {
        self.headers.insert(name, value);
    }

    pub fn set_content(&mut self, content: &[u8]) {
        self.content = Some(content.to_vec());
    }

    pub fn header_as_string(&self) -> String {
        let url = Url::new(&self.url).unwrap();
        let mut string = format!("{} {} {}\r\n", self.method, url.path(), self.version);
        for header in &self.headers {
            let (key, value) = header;
            string += &format!("{}: {}\r\n", key, value);
        }

        string
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header_as_string().into_bytes();

        bytes.push(b'\r');
        bytes.push(b'\n');

        if let Some(ref content) = self.content {
            bytes.extend_from_slice(content);
        }

        bytes
    }
}

#[derive(Clone, Debug)]
pub struct HttpResponse {
    version: String,
    status: Status,
    note: String,
    headers: HashMap<String, String>,
    content: Vec<u8>,
}

impl HttpResponse {
    pub fn statue(&self) -> Status {
        self.status
    }

    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }

    pub fn take(self) -> Vec<u8> {
        self.content
    }

    pub fn from_bytes(bytes: &[u8]) -> HttpResult<Self> {
        let (http_header, content) = Self::split_header_and_content(bytes)?;

        let http_header = str::from_utf8(http_header).map_err(|_| "invalid codepoint")?;
        let (response_line, headers_part) = http_header.split_once('\n').unwrap();
        let (version, remaining) = response_line
            .split_once(|c: char| c.is_ascii_whitespace())
            .ok_or("expected version")?;
        let (status, remaining) = remaining
            .split_once(|c: char| c.is_ascii_whitespace())
            .ok_or("expected status")?;
        let note = remaining;
        let headers = Self::parse_headers(headers_part)?;

        Ok(Self {
            version: version.trim().into(),
            status: status.trim().parse()?,
            note: note.trim().into(),
            content: if let Some(ref s) = headers.get("Transfer-Encoding")
                && s.as_str() == "chunked"
            {
                Self::load_chunk_content(content)
            } else {
                content.into()
            },
            headers,
        })
    }

    fn load_chunk_content(mut content: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();

        while !content.is_empty() {
            let mut iter = content.splitn(2, |b| b == &b'\n');
            if let Some(line_u8) = iter.next()
                && let Ok(line) = str::from_utf8(line_u8)
                && let Ok(len) = usize::from_str_radix(line.trim(), 16)
                && len != 0
                && let Some(slice) = content.get(..len)
            {
                content = iter.next().unwrap_or(&[]);
                if let Some(slice) = content.get(..len) {
                    bytes.extend_from_slice(slice);
                    content = &content[len..];
                }
            } else {
                break;
            }
        }

        bytes
    }

    fn split_header_and_content(bytes: &[u8]) -> HttpResult<(&[u8], &[u8])> {
        let mut i = 0;

        loop {
            if bytes.len() - 2 < i {
                break Err("invalid response");
            }
            if &bytes[i..i + 2] == b"\n\n" {
                break Ok(bytes.split_at(i + 1));
            }
            if i <= bytes.len() - 4 && &bytes[i..i + 4] == b"\r\n\r\n" {
                break Ok(bytes.split_at(i + 4));
            }
            i += 1;
        }
    }

    fn parse_headers(s: &str) -> HttpResult<HashMap<String, String>> {
        let mut headers = HashMap::new();

        for line in s.trim().split('\n') {
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            headers.insert(key.trim().into(), value.trim().into());
        }

        Ok(headers)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Continue = 100,
    SwitchingProtocols = 101,
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    TemporaryRedirect = 307,
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    RequestEntityTooLarge = 413,
    RequestUriTooLong = 414,
    UnsupportedMediaType = 415,
    RequestedRangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HttpVersionNotSupported = 505,
}

impl FromStr for Status {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, &'static str> {
        let value: u32 = s.parse().map_err(|_| "expected integer")?;
        value.try_into()
    }
}

impl TryFrom<u32> for Status {
    type Error = &'static str;

    fn try_from(value: u32) -> Result<Self, &'static str> {
        match value {
            100 => Ok(Self::Continue),
            101 => Ok(Self::SwitchingProtocols),
            200 => Ok(Self::Ok),
            201 => Ok(Self::Created),
            202 => Ok(Self::Accepted),
            203 => Ok(Self::NonAuthoritativeInformation),
            204 => Ok(Self::NoContent),
            205 => Ok(Self::ResetContent),
            206 => Ok(Self::PartialContent),
            300 => Ok(Self::MultipleChoices),
            301 => Ok(Self::MovedPermanently),
            302 => Ok(Self::Found),
            303 => Ok(Self::SeeOther),
            304 => Ok(Self::NotModified),
            305 => Ok(Self::UseProxy),
            307 => Ok(Self::TemporaryRedirect),
            400 => Ok(Self::BadRequest),
            401 => Ok(Self::Unauthorized),
            402 => Ok(Self::PaymentRequired),
            403 => Ok(Self::Forbidden),
            405 => Ok(Self::MethodNotAllowed),
            406 => Ok(Self::NotAcceptable),
            407 => Ok(Self::ProxyAuthenticationRequired),
            408 => Ok(Self::RequestTimeout),
            409 => Ok(Self::Conflict),
            410 => Ok(Self::Gone),
            411 => Ok(Self::LengthRequired),
            412 => Ok(Self::PreconditionFailed),
            413 => Ok(Self::RequestEntityTooLarge),
            414 => Ok(Self::RequestUriTooLong),
            415 => Ok(Self::UnsupportedMediaType),
            416 => Ok(Self::RequestedRangeNotSatisfiable),
            417 => Ok(Self::ExpectationFailed),
            500 => Ok(Self::InternalServerError),
            501 => Ok(Self::NotImplemented),
            502 => Ok(Self::BadGateway),
            503 => Ok(Self::ServiceUnavailable),
            504 => Ok(Self::GatewayTimeout),
            505 => Ok(Self::HttpVersionNotSupported),
            _ => Err("unknown status"),
        }
    }
}
