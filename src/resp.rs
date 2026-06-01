use thiserror::Error;

enum RESPData {
    SimpleString(String),
    Array(Vec<RESPData>),
}

impl From<RESPData> for String {
    fn from(data: RESPData) -> String {
        String::from("")
    }
}

fn parse_resp(input: &[u8]) -> anyhow::Result<RESPData> {
    let (res, _) = parse_resp_value(input, 0)?;
    Ok(res)
}

fn parse_resp_value(input: &[u8], _cursor: usize) -> anyhow::Result<(RESPData, usize)> {
    if _cursor >= input.len() {
        anyhow::bail!("Invalid cursor")
    }

    let mut cursor = _cursor;
    let value = input[cursor];
    match value {
        b'*' => {
            // array
            // find element count
            let count_start = cursor + 1;
            let mut count_end = count_start;

            while count_end < input.len() && input[count_end] != b'\r' {
                count_end += 1
            }

            let element_count_bytes = &input[count_start..count_end];
            let element_count_str = str::from_utf8(element_count_bytes)?;
            let element_count = element_count_str.parse::<usize>();

            cursor = count_end + 1;
            if cursor >= input.len() || input[cursor] != b'\n' {
                anyhow::bail!("invalid input")
            }
            cursor += 1;

            // start retrieving elements from cursor
            Ok((RESPData::Array(Vec::new()), cursor))
        }
        b'+' => {
            // simple string
            let str_start = cursor + 1;
            cursor = str_start;

            while cursor < input.len() && input[cursor] != b'\r' {
                cursor += 1
            }

            if cursor >= input.len() {
                anyhow::bail!("missing carriage return end")
            }

            // retrieve simple string
            let simple_string_bytes = &input[str_start..cursor];
            let simple_string = str::from_utf8(simple_string_bytes)?.to_string();

            // verify \r\n ending
            cursor += 1;
            if cursor >= input.len() || input[cursor] != b'\n' {
                anyhow::bail!("invalid input")
            }

            cursor += 1;

            Ok((RESPData::SimpleString(simple_string), cursor))
        }

        _ => {
            anyhow::bail!("invalid RESP data type")
        }
    }
}
