pub(super) fn extract_ret_fields(
    json: Option<&serde_json::Value>,
) -> (Option<String>, Option<String>) {
    let ret = json
        .and_then(|value| value.get("sys-header"))
        .and_then(|value| value.get("SYS_HEAD"))
        .and_then(|value| value.get("RET"))
        .and_then(|value| value.as_array())
        .and_then(|value| value.first());

    (
        ret.and_then(|value| value.get("RET_CODE"))
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned),
        ret.and_then(|value| value.get("RET_MSG"))
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned),
    )
}
