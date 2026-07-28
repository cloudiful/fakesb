use regex::Regex;
use serde_json::Value;

use crate::domain::{
    BodyFormat, BodyMatcher, MockRequest, RuleMatcher, StringMatcher, value_at_path,
};
use crate::service::ServiceError;

pub fn validate(matcher: &RuleMatcher) -> Result<(), ServiceError> {
    if matcher
        .method
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(ServiceError::Validation(
            "method matcher must not be empty".into(),
        ));
    }
    if matcher.path.is_some() && matcher.path_pattern.is_some() {
        return Err(ServiceError::Validation(
            "path and path_pattern cannot both be set".into(),
        ));
    }
    if let Some(pattern) = matcher.path_pattern.as_deref() {
        Regex::new(pattern)
            .map_err(|error| ServiceError::Validation(format!("invalid path pattern: {error}")))?;
    }
    for (name, value) in matcher.query.iter().chain(matcher.headers.iter()) {
        validate_string_matcher(name, value)?;
    }
    if let Some(body) = &matcher.body {
        validate_body(body)?;
    }
    Ok(())
}

pub fn matches(matcher: &RuleMatcher, request: &MockRequest) -> Result<bool, ServiceError> {
    if let Some(method) = &matcher.method {
        if !request.method.eq_ignore_ascii_case(method) {
            return Ok(false);
        }
    }
    if let Some(path) = &matcher.path {
        if request.path != *path {
            return Ok(false);
        }
    }
    if let Some(pattern) = &matcher.path_pattern {
        if !Regex::new(pattern)
            .map_err(|error| ServiceError::Validation(format!("invalid path pattern: {error}")))?
            .is_match(&request.path)
        {
            return Ok(false);
        }
    }
    for (name, expected) in &matcher.query {
        let Some(values) = request.query_values(name) else {
            return Ok(false);
        };
        let mut matched = false;
        for value in values {
            if string_matches(expected, value)? {
                matched = true;
                break;
            }
        }
        if !matched {
            return Ok(false);
        }
    }
    for (name, expected) in &matcher.headers {
        let Some(value) = request
            .headers
            .iter()
            .find(|(actual, _)| actual.eq_ignore_ascii_case(name))
            .map(|(_, value)| value)
        else {
            return Ok(false);
        };
        if !string_matches(expected, value)? {
            return Ok(false);
        }
    }
    matcher
        .body
        .as_ref()
        .map(|body| body_matches(body, request))
        .unwrap_or(Ok(true))
}

fn validate_string_matcher(name: &str, matcher: &StringMatcher) -> Result<(), ServiceError> {
    let count = [
        matcher.equal_to.is_some(),
        matcher.contains.is_some(),
        matcher.matches.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if count != 1 {
        return Err(ServiceError::Validation(format!(
            "matcher for {name} must define exactly one of equal_to, contains, or matches"
        )));
    }
    if let Some(pattern) = &matcher.matches {
        Regex::new(pattern)
            .map_err(|error| ServiceError::Validation(format!("invalid matcher regex: {error}")))?;
    }
    Ok(())
}

fn validate_body(body: &BodyMatcher) -> Result<(), ServiceError> {
    let count = [
        body.equal_to.is_some(),
        body.contains.is_some(),
        body.matches.is_some(),
        body.json_equal_to.is_some(),
        !body.fields.is_empty(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if count == 0 || count > 1 {
        return Err(ServiceError::Validation(
            "body matcher must define one matching strategy".into(),
        ));
    }
    if body.format != BodyFormat::Json && body.json_equal_to.is_some() {
        return Err(ServiceError::Validation(
            "json_equal_to requires a JSON body matcher".into(),
        ));
    }
    if body.format == BodyFormat::Text && !body.fields.is_empty() {
        return Err(ServiceError::Validation(
            "field matchers require a JSON or XML body matcher".into(),
        ));
    }
    if let Some(pattern) = &body.matches {
        Regex::new(pattern)
            .map_err(|error| ServiceError::Validation(format!("invalid body regex: {error}")))?;
    }
    for (path, matcher) in &body.fields {
        validate_string_matcher(path, matcher)?;
    }
    Ok(())
}

fn body_matches(body: &BodyMatcher, request: &MockRequest) -> Result<bool, ServiceError> {
    if body.format != request.body_format {
        return Ok(false);
    }
    if let Some(expected) = &body.equal_to {
        if body.format == BodyFormat::Xml {
            let expected = xml::convert::to_json(expected).map_err(|error| {
                ServiceError::Validation(format!("invalid expected XML: {error}"))
            })?;
            return Ok(request.body_value() == Some(&expected));
        }
        return Ok(request.raw_body == *expected);
    }
    if let Some(expected) = &body.contains {
        return Ok(request.raw_body.contains(expected));
    }
    if let Some(pattern) = &body.matches {
        return Ok(Regex::new(pattern)
            .map_err(|error| ServiceError::Validation(format!("invalid body regex: {error}")))?
            .is_match(&request.raw_body));
    }
    let Some(normalized) = request.body_value() else {
        return Ok(false);
    };
    if let Some(expected) = &body.json_equal_to {
        return Ok(normalized == expected);
    }
    for (path, expected) in &body.fields {
        let Some(value) = value_at_path(normalized, path) else {
            return Ok(false);
        };
        let value = value_to_match_string(value);
        if !string_matches(expected, &value)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn string_matches(matcher: &StringMatcher, actual: &str) -> Result<bool, ServiceError> {
    if let Some(expected) = &matcher.equal_to {
        return Ok(actual == expected);
    }
    if let Some(expected) = &matcher.contains {
        return Ok(actual.contains(expected));
    }
    if let Some(pattern) = &matcher.matches {
        return Ok(Regex::new(pattern)
            .map_err(|error| ServiceError::Validation(format!("invalid matcher regex: {error}")))?
            .is_match(actual));
    }
    Ok(false)
}

fn value_to_match_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{matches, validate};
    use crate::domain::{BodyFormat, BodyMatcher, MockRequest, RuleMatcher, StringMatcher};

    fn request() -> MockRequest {
        MockRequest {
            method: "POST".into(),
            path: "/payments".into(),
            query: BTreeMap::from([(String::from("mode"), vec![String::from("test")])]),
            query_string: "mode=test".into(),
            headers: BTreeMap::from([(
                String::from("content-type"),
                String::from("application/json"),
            )]),
            content_type: Some("application/json".into()),
            body_format: BodyFormat::Json,
            raw_body: r#"{"amount":42,"name":"Ada"}"#.into(),
            normalized_body: Some(serde_json::json!({"amount": 42, "name": "Ada"})),
        }
    }

    #[test]
    fn matches_generic_request_attributes() {
        let matcher = RuleMatcher {
            method: Some("post".into()),
            path: Some("/payments".into()),
            query: BTreeMap::from([(
                String::from("mode"),
                StringMatcher {
                    equal_to: Some("test".into()),
                    ..Default::default()
                },
            )]),
            headers: BTreeMap::from([(
                String::from("Content-Type"),
                StringMatcher {
                    contains: Some("json".into()),
                    ..Default::default()
                },
            )]),
            body: Some(BodyMatcher {
                format: BodyFormat::Json,
                json_equal_to: Some(serde_json::json!({"name": "Ada", "amount": 42})),
                ..Default::default()
            }),
            ..Default::default()
        };
        validate(&matcher).unwrap();
        assert!(matches(&matcher, &request()).unwrap());
    }

    #[test]
    fn rejects_multiple_body_strategies() {
        let matcher = RuleMatcher {
            body: Some(BodyMatcher {
                format: BodyFormat::Text,
                equal_to: Some("one".into()),
                contains: Some("o".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert!(validate(&matcher).is_err());
    }

    #[test]
    fn matches_text_body_without_a_normalized_value() {
        let request = MockRequest {
            method: "POST".into(),
            path: "/messages".into(),
            query: BTreeMap::new(),
            query_string: String::new(),
            headers: BTreeMap::new(),
            content_type: Some("text/plain".into()),
            body_format: BodyFormat::Text,
            raw_body: "hello world".into(),
            normalized_body: None,
        };

        for body in [
            BodyMatcher {
                format: BodyFormat::Text,
                equal_to: Some("hello world".into()),
                ..Default::default()
            },
            BodyMatcher {
                format: BodyFormat::Text,
                contains: Some("world".into()),
                ..Default::default()
            },
            BodyMatcher {
                format: BodyFormat::Text,
                matches: Some(r"^hello\s+world$".into()),
                ..Default::default()
            },
        ] {
            let matcher = RuleMatcher {
                body: Some(body),
                ..Default::default()
            };
            validate(&matcher).unwrap();
            assert!(matches(&matcher, &request).unwrap());
        }
    }

    #[test]
    fn matches_xml_semantically_across_formatting_changes() {
        let raw_body = "<order>\n  <id>7</id>\n</order>";
        let request = MockRequest {
            method: "POST".into(),
            path: "/orders".into(),
            query: BTreeMap::new(),
            query_string: String::new(),
            headers: BTreeMap::new(),
            content_type: Some("application/xml".into()),
            body_format: BodyFormat::Xml,
            raw_body: raw_body.into(),
            normalized_body: Some(xml::convert::to_json(raw_body).unwrap()),
        };
        let matcher = RuleMatcher {
            body: Some(BodyMatcher {
                format: BodyFormat::Xml,
                equal_to: Some("<order><id>7</id></order>".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        validate(&matcher).unwrap();
        assert!(matches(&matcher, &request).unwrap());
    }

    #[test]
    fn matches_path_patterns_and_repeated_query_values() {
        let mut request = request();
        request.path = "/payments/42".into();
        request
            .query
            .insert("tag".into(), vec!["first".into(), "second".into()]);
        let matcher = RuleMatcher {
            path_pattern: Some(r"^/payments/[0-9]+$".into()),
            query: BTreeMap::from([(
                "tag".into(),
                StringMatcher {
                    equal_to: Some("second".into()),
                    ..Default::default()
                },
            )]),
            ..Default::default()
        };

        validate(&matcher).unwrap();
        assert!(matches(&matcher, &request).unwrap());

        request.query.insert("tag".into(), vec!["first".into()]);
        assert!(!matches(&matcher, &request).unwrap());
    }

    #[test]
    fn rejects_invalid_path_pattern() {
        let matcher = RuleMatcher {
            path_pattern: Some("[".into()),
            ..Default::default()
        };

        assert!(validate(&matcher).is_err());
    }
}
