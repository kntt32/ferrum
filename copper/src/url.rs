use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Url<'a> {
    raw: &'a str,
    scheme: &'a str,
    host: &'a str,
    path: &'a str,
    query: &'a str,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_url() {
        assert_eq!(Url::new("http://example.com/").unwrap(), Url::EXAMPLE_COM);
    }
}

impl<'a> TryFrom<&'a str> for Url<'a> {
    type Error = String;

    fn try_from(value: &'a str) -> Result<Self, String> {
        Self::new(value)
    }
}

impl<'a> Url<'a> {
    pub const EXAMPLE_COM: Url<'a> = Self {
        raw: "http://example.com/",
        scheme: "http",
        host: "example.com",
        path: "/",
        query: "",
    };

    pub fn new(s: &'a str) -> Result<Url<'a>, String> {
        let raw = s.trim();
        let mut iter = raw.splitn(2, "://");
        let scheme = iter.next().ok_or("expected scheme")?;
        let mut iter = iter.next().ok_or("expected host")?.splitn(2, '/');
        let host = iter.next().ok_or("expectd host")?;
        let path;
        let query;
        if let Some(remaining) = iter.next() {
            let remaining = &raw[raw.len() - remaining.len() - 1..];
            let mut iter = remaining.splitn(2, '?');
            path = iter.next().ok_or("expected path")?;
            query = iter.next().unwrap_or("");
        } else {
            path = "/";
            query = "";
        }

        Ok(Self {
            raw,
            scheme,
            host,
            path,
            query,
        })
    }

    pub fn as_str(&self) -> &'a str {
        self.raw
    }

    pub fn scheme(&self) -> &'a str {
        self.scheme
    }

    pub fn host(&self) -> &'a str {
        self.host
    }

    pub fn path(&self) -> &'a str {
        self.path
    }

    pub fn query(&self) -> &'a str {
        self.query
    }
}
