use thiserror::Error;

#[derive(Debug)]
pub enum RESPData {
    SimpleString(String),
    Array(Vec<RESPData>),
    BulkString(Vec<u8>),
}

impl From<&RESPData> for Vec<u8> {
    fn from(data: &RESPData) -> Vec<u8> {
        match data {
            RESPData::SimpleString(ss) => format!("+{}\r\n", ss).as_bytes().to_vec(),
            RESPData::BulkString(bulk_str) => {
                let mut res = Vec::new();
                res.extend_from_slice(b"$");
                res.extend(bulk_str.len().to_string().as_bytes());
                res.extend(b"\r\n");
                res.extend(bulk_str);
                res.extend(b"\r\n");
                res
            }
            _ => Vec::new(),
        }
    }
}

pub fn parse_resp(input: &[u8]) -> anyhow::Result<RESPData> {
    let (res, _) = parse_resp_value(input, 0)?;
    Ok(res)
}

fn parse_resp_value(input: &[u8], starting_cursor: usize) -> anyhow::Result<(RESPData, usize)> {
    if starting_cursor >= input.len() {
        anyhow::bail!("Invalid cursor")
    }

    let mut cursor = starting_cursor;
    let value = input[cursor];
    match value {
        b'*' => {
            // array
            cursor += 1;
            let (element_count_bytes, new_cursor) = read_bytes_until_crlf(input, cursor)?;
            cursor = new_cursor;
            let element_count_str = str::from_utf8(element_count_bytes)?;
            let element_count = element_count_str.parse::<usize>()?;

            // create result
            let mut res_vec = Vec::<RESPData>::with_capacity(element_count);

            // start retrieving elements from cursor
            for _ in 0..element_count {
                let (resp_data, new_cursor) = parse_resp_value(input, cursor)?;
                res_vec.push(resp_data);
                cursor = new_cursor;
            }

            Ok((RESPData::Array(res_vec), cursor))
        }
        b'+' => {
            // simple string
            cursor += 1;
            let (str_bytes, cursor) = read_bytes_until_crlf(input, cursor)?;
            let simple_string = str::from_utf8(str_bytes)?.to_string();

            Ok((RESPData::SimpleString(simple_string), cursor))
        }
        b'$' => {
            // bulk string
            cursor += 1;
            let (str_len_bytes, new_cursor) = read_bytes_until_crlf(input, cursor)?;
            cursor = new_cursor;

            let str_len = str::from_utf8(str_len_bytes)?.parse::<usize>()?;
            if cursor + str_len + 2 > input.len() {
                anyhow::bail!("Bulk string length exceeded input")
            }

            let bulk_str = input[cursor..cursor + str_len].to_vec();

            // verify \r\n
            cursor += str_len;
            if input[cursor] != b'\r' || input[cursor + 1] != b'\n' {
                anyhow::bail!("Missing carriage return")
            }
            cursor += 2;

            Ok((RESPData::BulkString(bulk_str), cursor))
        }
        _ => {
            anyhow::bail!("invalid RESP data type")
        }
    }
}

pub fn read_bytes_until_crlf(
    input: &[u8],
    starting_cursor: usize,
) -> anyhow::Result<(&[u8], usize)> {
    let mut cursor = starting_cursor;
    let bytes_start = cursor;
    while cursor < input.len() && input[cursor] != b'\r' {
        cursor += 1
    }

    if cursor >= input.len() {
        anyhow::bail!("missing carriage return");
    }

    let res = &input[bytes_start..cursor];

    // verify \r\n ending
    cursor += 1;
    if cursor >= input.len() || input[cursor] != b'\n' {
        anyhow::bail!("missing carriage return")
    }

    // advance cursor to after crlf
    cursor += 1;

    Ok((res, cursor))
}
