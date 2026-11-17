// `pocketstorage:empty_psu`: the PSU item the upgrade recipe names.
fn pocketstorage_ingredient(
    kind: &str,
    data: &serde_json::Map<String, serde_json::Value>,
    items: &mut Vec<String>,
) {
    if kind == "pocketstorage:empty_psu"
        && let Some(item) = data.get("item").and_then(|v| v.as_str())
    {
        items.push(item.to_owned());
    }
}
