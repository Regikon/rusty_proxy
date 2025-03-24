use std::{fmt, str::from_utf8};

use super::primitives::{parse_token, parse_uri};
use bytes::Bytes;
use http::{Method, Uri, Version};

const HTTP_11: &[u8] = b"HTTP 1.1";
const HTTP_10: &[u8] = b"HTTP 1.0";
const HTTP_VER_LEN: usize = 8;

pub struct RequestLine {
    pub method: Method,
    pub uri: Uri,
    pub version: Version,
}

fn parse_request_line(request_line: Bytes) -> Result<RequestLine, String> {
    // Request-Line = Method SP Request-URI SP HTTP-Version CRLF
    let method_end = match parse_token(&request_line) {
        Ok(0) => {
            return Err(String::from(
                "unexpected end of line when parsing request method",
            ))
        }
        Ok(size) => size,
        Err(_) => return Err(String::from("failed when parsing request method")),
    };
    let method = match Method::from_bytes(&request_line[0..=method_end]) {
        Ok(method) => method,
        Err(_) => return Err(String::from("got non-standard request header")),
    };

    if method_end + 2 >= request_line.len() {
        return Err(String::from(
            "unexpected end of line when parsing request URI",
        ));
    }
    let request_line = request_line.slice((method_end + 2)..);

    let uri_end = parse_uri(&request_line)?;
    let uri = match Uri::from_maybe_shared(request_line.slice(0..=uri_end)) {
        Ok(uri) => uri,
        Err(_) => {
            return Err(fmt::format(format_args!(
                "invalid uri: {}",
                from_utf8(&request_line[0..=uri_end]).unwrap(),
            )))
        }
    };

    if uri_end + 1 + HTTP_VER_LEN > request_line.len() {
        return Err(String::from(
            "unexpected end of line when parsing HTTP version",
        ));
    }
    let request_line = &request_line[(uri_end + 2)..];

    let version_bytes = &request_line[0..HTTP_VER_LEN];
    let version: http::Version;
    if version_bytes == HTTP_10 {
        version = Version::HTTP_10;
    } else if version_bytes == HTTP_11 {
        version = Version::HTTP_11;
    } else {
        return Err(fmt::format(format_args!(
            "unsupported protocol version: {}",
            from_utf8(version_bytes).unwrap()
        )));
    };

    return Ok(RequestLine {
        method,
        uri,
        version,
    });
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_request_line() {
        let line = b"GET https://docs.rs/bytes/1.10.1/bytes/struct.Bytes.html HTTP 1.1";
        let uri = "https://docs.rs/bytes/1.10.1/bytes/struct.Bytes.html"
            .parse::<Uri>()
            .unwrap();
        let result = parse_request_line(Bytes::from_static(line)).unwrap();
        assert_eq!(result.uri, uri);
        assert_eq!(result.method, http::Method::GET);
        assert_eq!(result.version, http::Version::HTTP_11);
    }
}
