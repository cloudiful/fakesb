use roxmltree::Node;

pub struct XPath {
    steps: Vec<Step>,
}

struct Step {
    name: Option<String>,
    index: Option<usize>,
    attribute: bool,
    text: bool,
    descendant: bool,
}

impl XPath {
    pub fn parse(expression: &str) -> Result<Self, String> {
        let trimmed = expression.trim();
        if trimmed.is_empty() || !trimmed.starts_with('/') {
            return Err(format!("xpath must start with '/': {expression}"));
        }
        let mut steps = Vec::new();
        let mut descendant = trimmed.starts_with("//");
        let body = trimmed.trim_start_matches('/');
        for raw_step in body.split('/') {
            let raw_step = raw_step.trim();
            if raw_step.is_empty() {
                continue;
            }
            let (name_part, index_part) = split_index(raw_step)?;
            let attribute = name_part.starts_with('@');
            let name = if attribute {
                name_part
                    .strip_prefix('@')
                    .map(str::to_string)
                    .filter(|name| name != "*")
            } else {
                (name_part != "*" && name_part != "text()").then(|| name_part.to_string())
            };
            let text = name_part == "text()";
            let index = index_part.as_deref().map(parse_index).transpose()?;
            steps.push(Step {
                name,
                index,
                attribute,
                text,
                descendant,
            });
            descendant = false;
        }
        if steps.is_empty() {
            return Err(format!("xpath has no steps: {expression}"));
        }
        Ok(Self { steps })
    }

    pub fn evaluate(&self, xml: &str) -> Result<Vec<String>, String> {
        let document = roxmltree::Document::parse(xml)
            .map_err(|error| format!("invalid XML body: {error}"))?;
        let mut current = vec![document.root_element()];
        for (step_index, step) in self.steps.iter().enumerate() {
            if step.descendant {
                let mut expanded = Vec::new();
                for node in current {
                    expanded.extend(collect_descendants(node));
                }
                current = expanded;
            }
            if step.attribute || step.text {
                continue;
            }
            let mut next = Vec::new();
            if step_index == 0 || step.descendant {
                next = current
                    .into_iter()
                    .filter(|node| step.matches(*node))
                    .collect();
            } else {
                for node in current {
                    for child in node.children().filter(|child| child.is_element()) {
                        if step.matches(child) {
                            next.push(child);
                        }
                    }
                }
            }
            current = next;
        }

        let mut values = Vec::new();
        for node in current {
            let last = self.steps.last().expect("steps is non-empty");
            if last.attribute {
                for attribute in node.attributes() {
                    if last
                        .name
                        .as_deref()
                        .is_none_or(|name| attribute.name() == name)
                    {
                        values.push(attribute.value().to_string());
                    }
                }
            } else if last.text {
                if let Some(text) = node.text().map(str::trim).filter(|text| !text.is_empty()) {
                    values.push(text.to_string());
                }
            } else {
                let text = text_content(node);
                if !text.is_empty() {
                    values.push(text);
                }
            }
        }
        Ok(values)
    }
}

impl Step {
    fn matches(&self, node: Node<'_, '_>) -> bool {
        if self.attribute {
            return false;
        }
        if self
            .name
            .as_deref()
            .is_some_and(|name| node.tag_name().name() != name)
        {
            return false;
        }
        if let Some(index) = self.index {
            let position = node
                .parent()
                .and_then(|parent| {
                    parent
                        .children()
                        .filter(|child| child.is_element())
                        .position(|child| child == node)
                })
                .map(|position| position + 1);
            if position != Some(index) {
                return false;
            }
        }
        true
    }
}

fn collect_descendants<'a, 'b>(node: Node<'a, 'b>) -> Vec<Node<'a, 'b>> {
    node.descendants()
        .filter(|child| child.is_element())
        .collect()
}

fn text_content(node: Node<'_, '_>) -> String {
    node.text()
        .map(|text| text.trim().to_string())
        .or_else(|| {
            node.children()
                .filter(|child| child.is_text())
                .map(|child| child.text().unwrap_or_default().trim().to_string())
                .filter(|text| !text.is_empty())
                .reduce(|left, right| format!("{left}{right}"))
        })
        .unwrap_or_default()
}

fn split_index(step: &str) -> Result<(String, Option<String>), String> {
    if let Some(start) = step.rfind('[') {
        if !step.ends_with(']') {
            return Err(format!("malformed index in xpath step: {step}"));
        }
        Ok((
            step[..start].to_string(),
            Some(step[start + 1..step.len() - 1].to_string()),
        ))
    } else {
        Ok((step.to_string(), None))
    }
}

fn parse_index(raw: &str) -> Result<usize, String> {
    let index = raw
        .parse::<usize>()
        .map_err(|_| format!("invalid xpath index: [{raw}]"))?;
    if index < 1 {
        return Err(format!("xpath index must be 1-based: [{raw}]"));
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::XPath;

    const XML: &str = r#"
        <order version="2">
          <customer>
            <name>Ada</name>
          </customer>
          <items>
            <item>
              <id>1</id>
              <sku>A-1</sku>
            </item>
            <item>
              <id>2</id>
              <sku>B-2</sku>
            </item>
          </items>
          <remark>first</remark>
          <remark>second</remark>
        </order>
    "#;

    fn values(expression: &str) -> Vec<String> {
        XPath::parse(expression)
            .expect("expression should parse")
            .evaluate(XML)
            .expect("xml should evaluate")
    }

    #[test]
    fn reads_single_text_value() {
        assert_eq!(values("/order/customer/name"), ["Ada"]);
    }

    #[test]
    fn reads_repeated_elements() {
        assert_eq!(values("/order/items/item/id"), ["1", "2"]);
    }

    #[test]
    fn reads_repeated_siblings() {
        assert_eq!(values("/order/remark"), ["first", "second"]);
    }

    #[test]
    fn reads_attributes() {
        assert_eq!(values("/order/@version"), ["2"]);
    }

    #[test]
    fn finds_descendants() {
        assert_eq!(values("//id"), ["1", "2"]);
        assert_eq!(values("//item/sku"), ["A-1", "B-2"]);
    }

    #[test]
    fn supports_indexing() {
        assert_eq!(values("/order/items/item[2]/id"), ["2"]);
    }

    #[test]
    fn reads_text_node_selector() {
        assert_eq!(values("/order/customer/name/text()"), ["Ada"]);
    }

    #[test]
    fn rejects_malformed_expressions() {
        assert!(XPath::parse("order/name").is_err());
        assert!(XPath::parse("/order/item[0]").is_err());
        assert!(XPath::parse("/order/item[x]").is_err());
    }
}
