use crate::domain::ParsedEsbMessage;
use crate::service::ServiceError;

pub fn parse_request(raw_body: &str) -> Result<ParsedEsbMessage, ServiceError> {
    let json = xml::convert::to_json(raw_body)
        .map_err(|err| ServiceError::Parse(format!("failed to parse xml request: {err}")))?;
    let simplified = crate::http::json::simplify(&json);
    Ok(ParsedEsbMessage {
        service_code: read_string_path(&simplified, &["sys-header", "SYS_HEAD", "SERVICE_CODE"])?,
        message_type: read_string_path(&simplified, &["sys-header", "SYS_HEAD", "MESSAGE_TYPE"])?,
        message_code: read_string_path(&simplified, &["sys-header", "SYS_HEAD", "MESSAGE_CODE"])?,
        normalized_json: simplified,
        raw_body: raw_body.to_string(),
    })
}

fn read_string_path(value: &serde_json::Value, path: &[&str]) -> Result<String, ServiceError> {
    let mut current = value;
    for segment in path {
        current = current.get(*segment).ok_or_else(|| {
            ServiceError::Parse(format!("missing request field: {}", path.join(".")))
        })?;
    }

    current.as_str().map(ToOwned::to_owned).ok_or_else(|| {
        ServiceError::Parse(format!("request field is not string: {}", path.join(".")))
    })
}

#[cfg(test)]
mod tests {
    use super::{parse_request, read_string_path};

    #[test]
    fn xml_without_service_field_is_a_parse_error() {
        let error = parse_request(
            "<request><sys-header><SYS_HEAD><MESSAGE_TYPE>type</MESSAGE_TYPE><MESSAGE_CODE>code</MESSAGE_CODE></SYS_HEAD></sys-header></request>",
        )
        .expect_err("missing service code must fail");
        assert!(error.to_string().contains("SERVICE_CODE"));
    }

    #[test]
    fn missing_service_field_is_a_parse_error() {
        let value = serde_json::json!({
            "sys-header": { "SYS_HEAD": { "MESSAGE_TYPE": "type", "MESSAGE_CODE": "code" } }
        });
        let error = read_string_path(&value, &["sys-header", "SYS_HEAD", "SERVICE_CODE"])
            .expect_err("missing service code must fail");
        assert!(error.to_string().contains("SERVICE_CODE"));
    }
}
