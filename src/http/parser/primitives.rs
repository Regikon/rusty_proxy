use http::{HeaderName, HeaderValue};

const SEPARATORS: &'static [u8] = b"()<>@,;:\\\"/[]?={} \t";
const CHAR_MAX: u8 = 127;

#[inline]
fn is_linear_whitespace(symbol: u8) -> bool {
    symbol == b'\t' || symbol == b' '
}

fn is_separator(symbol: u8) -> bool {
    SEPARATORS.contains(&symbol)
}

#[inline]
fn is_part_of_text(symbol: u8) -> bool {
    !symbol.is_ascii_control() || is_linear_whitespace(symbol)
}

#[inline]
fn is_part_of_token(symbol: u8) -> bool {
    symbol.is_ascii() && !symbol.is_ascii_control() && !is_separator(symbol)
}

#[inline]
fn is_part_of_uri(symbol: u8) -> bool {
    symbol.is_ascii() && !symbol.is_ascii_control() && !is_linear_whitespace(symbol)
}

pub fn parse_comment(comment_line: &[u8]) -> Result<usize, String> {
    // comment  = "(" *( ctext | quoted-pair | comment ) ")"
    // ctext = <any TEXT excluding "(" and ")">
    // quoted-pair = "\" CHAR

    if comment_line.len() < 2 {
        return Err(String::from(
            "passed comment line is empty, expected at least ()",
        ));
    }
    if comment_line[0] != b'(' {
        return Err(String::from(
            "passed line does not starts with open parenthesis (",
        ));
    }
    let mut parenthesis_counter = 0;
    let mut skip_next = false;
    for (idx, &byte) in comment_line.iter().enumerate() {
        if skip_next {
            skip_next = false;
            // We should skip only if next byte is a CHAR (0 ..=127)
            if byte <= CHAR_MAX {
                continue;
            }
        }

        match byte {
            b'(' => {
                parenthesis_counter += 1;
            }
            b')' => {
                parenthesis_counter -= 1;
                if parenthesis_counter == 0 {
                    return Ok(idx);
                }
            }
            b'\\' => {
                skip_next = true;
            }
            text_byte => {
                if !is_part_of_text(text_byte) {
                    return Err(String::from("unexpected control sequence inside a comment"));
                }
            }
        }
    }
    // parenthesis_counter is not zero, so comment is not closed
    // it is an obvious error
    Err(String::from("unexpected end of line, expected )"))
}

pub fn parse_token(token_line: &[u8]) -> Result<usize, String> {
    if token_line.len() == 0 {
        return Err(String::from("empty token"));
    }
    for (idx, &byte) in token_line.iter().enumerate() {
        if !is_part_of_token(byte) {
            return Ok(idx - 1);
        }
    }
    Ok(token_line.len() - 1)
}

pub fn parse_uri(uri_line: &[u8]) -> Result<usize, String> {
    if uri_line.len() == 0 {
        return Err(String::from("empty uri"));
    }
    for (idx, &byte) in uri_line.iter().enumerate() {
        if !is_part_of_uri(byte) {
            if idx < 1 {
                return Err(String::from("empty uri"));
            }
            return Ok(idx - 1);
        }
    }
    Ok(uri_line.len() - 1)
}

pub fn parse_header(header_line: &[u8]) -> Result<(HeaderName, HeaderValue), String> {
    // message-header = field-name ":" [field-value]
    // field-name = token
    // field-value = *( field-content | LWS )
    // field-content = <*TEXT or combinations of token, separators and quoted-string>

    let name_end = parse_token(header_line)?;
    if name_end == header_line.len() - 1 || header_line[name_end + 1] != b':' {
        return Err(String::from(
            "unexpected symbol or eol after field name, expected :",
        ));
    }
    let header_name = match HeaderName::from_bytes(&header_line[0..=name_end]) {
        Ok(name) => name,
        Err(_) => return Err(String::from("failed to parse header name")),
    };

    // HTTP RFC states that there may be leading spaces before header value, so we skip them
    let mut value_start = name_end + 2;
    while value_start < header_line.len() && is_linear_whitespace(header_line[value_start]) {
        value_start += 1;
    }

    let header_value = &header_line[value_start..];
    let header_value = match HeaderValue::from_bytes(&header_value) {
        Ok(value) => value,
        Err(_) => return Err(String::from("failed to parse header value")),
    };

    Ok((header_name, header_value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_comment() {
        let positive_tests: Vec<(&[u8], usize)> = vec![
            (b"(Some simple comment)12345", 20),
            (b"((()))", 5),
            (b"(\\n\\n)", 5),
            (b"(\\t\t long     comment (\\\\))", 26),
            (b"()", 1),
        ];
        for (bytes, expected) in positive_tests {
            let result = parse_comment(bytes).unwrap();
            assert_eq!(result, expected);
        }

        let empty_comment_err = String::from("passed comment line is empty, expected at least ()");

        let unexpected_control_sequence =
            String::from("unexpected control sequence inside a comment");

        let unexpected_eol = String::from("unexpected end of line, expected )");

        let wrong_start = String::from("passed line does not starts with open parenthesis (");

        let negative_tests: Vec<(&[u8], &String)> = vec![
            (b"", &empty_comment_err),
            (b"(", &empty_comment_err),
            (b"g()", &wrong_start),
            (b" ()", &wrong_start),
            (b"(\n)", &unexpected_control_sequence),
            (b"(\r)", &unexpected_control_sequence),
            (b"(()", &unexpected_eol),
            (b"(Some pretty valid comment \t ()", &unexpected_eol),
        ];

        for (bytes, expected_error) in negative_tests {
            let result = parse_comment(bytes).unwrap_err();
            assert_eq!(result, *expected_error);
        }
    }
    #[test]
    fn test_parse_header() {
        let positive_tests: Vec<(&[u8], &str, &str)> = vec![
            (
                b"Accept-Encoding:    gzip, deflate",
                "accept-encoding",
                "gzip, deflate",
            ),
            (b"Referer:", "referer", ""),
            (b"Referer:         ", "referer", ""),
        ];
        positive_tests
            .iter()
            .for_each(|(input, expected_name, expected_value)| {
                let (name, value) = parse_header(input).unwrap();
                assert_eq!(name.as_str(), *expected_name);
                assert_eq!(value.to_str().unwrap(), *expected_value);
            });
    }
}
