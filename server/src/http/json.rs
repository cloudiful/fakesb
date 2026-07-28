use serde_json::{Map, Value};

const HEADER_SECTIONS: &[&str] = &["app-header", "local-header", "sys-header", "body"];
const NAME_KEY: &str = "@name";
const TYPE_KEY: &str = "@type";
const VALUE_KEY: &str = "$value";
const DATA_KEY: &str = "data";
const STRUCT_KEY: &str = "struct";
const FIELD_KEY: &str = "field";
const ARRAY_KEY: &str = "array";
const NUMBER_TYPE: &str = "number";

pub fn simplify(input: &Value) -> Value {
    JsonSimplifier::simplify(input)
}

#[derive(Debug, Clone)]
enum DataType {
    Object(Map<String, Value>),
    Array(Vec<Value>),
    Other,
}

impl From<&Value> for DataType {
    fn from(value: &Value) -> Self {
        match value {
            Value::Object(map) => DataType::Object(map.clone()),
            Value::Array(arr) => DataType::Array(arr.clone()),
            _ => DataType::Other,
        }
    }
}

struct JsonSimplifier;

impl JsonSimplifier {
    fn simplify(input: &Value) -> Value {
        match input {
            Value::Object(map) => {
                let mut result = Map::new();
                for (key, value) in map {
                    let processed = if HEADER_SECTIONS.contains(&key.as_str()) {
                        Self::simplify_section(value)
                    } else {
                        Self::simplify(value)
                    };
                    result.insert(key.clone(), processed);
                }
                Value::Object(result)
            }
            _ => input.clone(),
        }
    }

    fn simplify_section(section: &Value) -> Value {
        if let Value::Object(map) = section {
            if let Some(data) = map.get(DATA_KEY) {
                return match DataType::from(data) {
                    DataType::Object(data_obj) => Self::process_object_data(&data_obj, section),
                    DataType::Array(_) => Self::process_data_array(data),
                    DataType::Other => Self::simplify(section),
                };
            }
        }
        section.clone()
    }

    fn process_object_data(data_obj: &Map<String, Value>, fallback_section: &Value) -> Value {
        if let Some(name) = data_obj.get(NAME_KEY) {
            let mut result = Map::new();
            if let Some(struct_data) = data_obj.get(STRUCT_KEY).and_then(|s| s.get(DATA_KEY)) {
                result.insert(
                    Self::get_string_value(name),
                    Self::process_data_array(struct_data),
                );
            }
            Value::Object(result)
        } else {
            Self::simplify(fallback_section)
        }
    }

    fn process_item_data(item_map: &Map<String, Value>, name: &str) -> Value {
        if let Some(field) = item_map.get(FIELD_KEY) {
            Self::extract_field_value(field)
        } else if let Some(array) = item_map.get(ARRAY_KEY) {
            Self::process_array_field(array, name)
        } else {
            Value::String(String::new())
        }
    }

    fn process_array_field(array: &Value, field_name: &str) -> Value {
        array
            .get(STRUCT_KEY)
            .map(|struct_data| match DataType::from(struct_data) {
                DataType::Object(struct_obj) => {
                    Self::process_single_struct(&struct_obj, field_name)
                }
                DataType::Array(struct_array) => {
                    Self::process_struct_array(&struct_array, field_name)
                }
                DataType::Other => Value::Array(Vec::new()),
            })
            .unwrap_or_else(|| Value::Array(Vec::new()))
    }

    fn process_single_struct(struct_obj: &Map<String, Value>, field_name: &str) -> Value {
        struct_obj
            .get(DATA_KEY)
            .map(|data| match DataType::from(data) {
                DataType::Array(data_array) => {
                    let item_map = Self::process_struct_data_array(&data_array);
                    Value::Array(vec![Value::Object(item_map)])
                }
                DataType::Object(_) => Self::process_single_data_object(data, field_name),
                DataType::Other => Value::Array(Vec::new()),
            })
            .unwrap_or_else(|| Value::Array(Vec::new()))
    }

    fn process_single_data_object(data: &Value, field_name: &str) -> Value {
        if let Some(field) = data.get(FIELD_KEY) {
            let value = Self::extract_field_value(field);
            let mut item = Map::new();
            item.insert(field_name.to_string(), value);
            Value::Array(vec![Value::Object(item)])
        } else {
            Value::Array(Vec::new())
        }
    }

    fn process_struct_array(struct_array: &[Value], field_name: &str) -> Value {
        Value::Array(
            struct_array
                .iter()
                .filter_map(|struct_item| {
                    struct_item
                        .get(DATA_KEY)
                        .and_then(|data| match DataType::from(data) {
                            DataType::Array(data_array) => {
                                Some(Value::Object(Self::process_struct_data_array(&data_array)))
                            }
                            DataType::Object(data_obj) => data_obj.get(FIELD_KEY).map(|field| {
                                let value = Self::extract_field_value(field);
                                let mut item = Map::new();
                                item.insert(field_name.to_string(), value);
                                Value::Object(item)
                            }),
                            DataType::Other => None,
                        })
                })
                .collect(),
        )
    }

    fn process_data_array(data: &Value) -> Value {
        if let Value::Array(arr) = data {
            let mut result = Map::new();
            for item in arr {
                if let Value::Object(item_map) = item {
                    if let Some(name) = Self::get_name_from_object(item_map) {
                        result.insert(name.clone(), Self::process_item_data(item_map, &name));
                    }
                }
            }
            Value::Object(result)
        } else {
            data.clone()
        }
    }

    fn process_struct_data_array(data_array: &[Value]) -> Map<String, Value> {
        let mut item_map = Map::new();
        for data_item in data_array {
            if let Value::Object(data_obj) = data_item {
                if let Some(name) = Self::get_name_from_object(data_obj) {
                    item_map.insert(name.clone(), Self::process_item_data(data_obj, &name));
                }
            }
        }
        item_map
    }

    fn extract_field_value(field: &Value) -> Value {
        if let Some(value_str) = field.get(VALUE_KEY) {
            let field_type = field.get(TYPE_KEY).and_then(|t| t.as_str());
            match field_type {
                Some(NUMBER_TYPE) => Self::parse_number_value(value_str),
                _ => Value::String(Self::get_string_value(value_str)),
            }
        } else {
            Value::String(String::new())
        }
    }

    fn get_name_from_object(obj: &Map<String, Value>) -> Option<String> {
        obj.get(NAME_KEY)
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
    }

    fn get_string_value(value: &Value) -> String {
        value.as_str().unwrap_or("").to_string()
    }

    fn parse_number_value(value_str: &Value) -> Value {
        if let Some(num_str) = value_str.as_str() {
            if let Ok(num) = num_str.parse::<i64>() {
                return Value::Number(serde_json::Number::from(num));
            }
            if let Ok(num) = num_str.parse::<f64>() {
                if let Some(json_num) = serde_json::Number::from_f64(num) {
                    return Value::Number(json_num);
                }
            }
        }
        Value::String(Self::get_string_value(value_str))
    }
}
