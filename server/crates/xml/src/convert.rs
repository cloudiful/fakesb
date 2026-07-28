use anyhow::{Context, Result};
use serde_json::{Map, Value};
use std::collections::HashMap;

pub struct XmlToJsonConverter;

impl XmlToJsonConverter {
    pub fn convert(xml: &str) -> Result<Value> {
        if xml.trim().is_empty() {
            return Ok(Value::default());
        }

        let doc = roxmltree::Document::parse(xml).context("failed to parse XML document")?;
        let root = doc.root_element();
        Self::node_to_json_value(root)
    }

    fn node_to_json_value(node: roxmltree::Node<'_, '_>) -> Result<Value> {
        let element_info = ElementInfo::from_node(node);

        match element_info.classify() {
            ElementType::Empty => Ok(Value::Object(Map::new())),
            ElementType::TextOnly => Ok(Value::String(element_info.text_content)),
            ElementType::Complex => Self::build_complex_object(node, element_info),
        }
    }

    fn build_complex_object(
        node: roxmltree::Node<'_, '_>,
        element_info: ElementInfo<'_, '_>,
    ) -> Result<Value> {
        let mut json_object = Map::new();

        for attr in node.attributes() {
            let key = format!("@{}", attr.name());
            let value = Self::escape_json_string(attr.value());
            json_object.insert(key, Value::String(value));
        }

        if !element_info.text_content.is_empty() && element_info.has_mixed_content() {
            json_object.insert(
                "$value".to_string(),
                Value::String(element_info.text_content),
            );
        }

        for (tag_name, children) in element_info.children_map {
            let json_value = if children.len() == 1 {
                Self::node_to_json_value(children[0])?
            } else {
                let array: Result<Vec<Value>> =
                    children.into_iter().map(Self::node_to_json_value).collect();
                Value::Array(array?)
            };

            json_object.insert(tag_name, json_value);
        }

        Ok(Value::Object(json_object))
    }

    fn escape_json_string(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
            .replace('"', "\\\"")
    }
}

struct ElementInfo<'a, 'input> {
    text_content: String,
    children_map: HashMap<String, Vec<roxmltree::Node<'a, 'input>>>,
    attribute_count: usize,
    element_count: usize,
}

impl<'a, 'input> ElementInfo<'a, 'input> {
    fn from_node(node: roxmltree::Node<'a, 'input>) -> Self {
        let mut info = Self {
            text_content: String::new(),
            children_map: HashMap::new(),
            attribute_count: node.attributes().count(),
            element_count: 0,
        };

        for child in node.children() {
            if child.is_element() {
                let tag_name = child.tag_name().name().to_string();
                info.children_map.entry(tag_name).or_default().push(child);
                info.element_count += 1;
            } else if child.is_text() {
                if let Some(text) = child.text() {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        info.text_content.push_str(trimmed);
                    }
                }
            }
        }

        info
    }

    fn classify(&self) -> ElementType {
        if self.is_empty() {
            ElementType::Empty
        } else if self.is_text_only() {
            ElementType::TextOnly
        } else {
            ElementType::Complex
        }
    }

    fn is_empty(&self) -> bool {
        self.attribute_count == 0 && self.element_count == 0 && self.text_content.is_empty()
    }

    fn is_text_only(&self) -> bool {
        self.attribute_count == 0 && self.element_count == 0 && !self.text_content.is_empty()
    }

    fn has_mixed_content(&self) -> bool {
        !self.text_content.is_empty() && (self.attribute_count > 0 || self.element_count > 0)
    }
}

#[derive(Debug, PartialEq)]
enum ElementType {
    Empty,
    TextOnly,
    Complex,
}

pub fn to_json(xml: &str) -> Result<Value> {
    XmlToJsonConverter::convert(xml)
}

#[cfg(test)]
mod tests {
    use super::to_json;
    use serde_json::json;

    #[test]
    fn converts_esb_headers_and_body() {
        let result = to_json(
            r#"<request><sys-header><SYS_HEAD><SERVICE_CODE>SVC</SERVICE_CODE><MESSAGE_TYPE>TYPE</MESSAGE_TYPE><MESSAGE_CODE>CODE</MESSAGE_CODE></SYS_HEAD></sys-header><body><FIELD>value</FIELD></body></request>"#,
        )
        .unwrap();

        assert_eq!(result["sys-header"]["SYS_HEAD"]["SERVICE_CODE"], "SVC");
        assert_eq!(result["body"]["FIELD"], "value");
    }

    #[test]
    fn preserves_attributes_and_numeric_metadata() {
        let result = to_json(r#"<field type="number">42</field>"#).unwrap();

        assert_eq!(result, json!({"@type": "number", "$value": "42"}));
    }

    #[test]
    fn groups_repeated_elements_as_arrays() {
        let result = to_json(r#"<items><item>one</item><item>two</item></items>"#).unwrap();

        assert_eq!(result["item"], json!(["one", "two"]));
    }

    #[test]
    fn represents_empty_elements_as_objects() {
        assert_eq!(to_json("<empty/>").unwrap(), json!({}));
        assert_eq!(to_json("").unwrap(), json!(null));
    }

    #[test]
    fn rejects_invalid_xml() {
        assert!(to_json("<request>").is_err());
    }
}
