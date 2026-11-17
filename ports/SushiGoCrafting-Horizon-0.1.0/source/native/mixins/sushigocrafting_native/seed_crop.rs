// Each seed is a copy of wheat seeds (rice) or beetroot seeds (the others), a BlockItem of
// `minecraft:wheat` / `minecraft:beetroots`: its row is pointed at its own crop, so planting it
// places the crop. Runs inside `J::rename` (every node of every copied table), so it returns at
// once for anything that isn't one of these seeds. Self-contained: `mod_content.rs` is built into
// `horizon_client_lib` (no `crate::game` there). Same table as `crop_seed.rs`.
fn sushigocrafting_seed_crop(row: &mut J, rename: &Rename) {
    const PAIRS: [(&str, &str); 5] = [
        ("sushigocrafting:rice_crop", "sushigocrafting:rice_seeds"),
        (
            "sushigocrafting:cucumber_crop",
            "sushigocrafting:cucumber_seeds",
        ),
        ("sushigocrafting:soy_crop", "sushigocrafting:soy_seeds"),
        (
            "sushigocrafting:wasabi_crop",
            "sushigocrafting:wasabi_seeds",
        ),
        (
            "sushigocrafting:sesame_crop",
            "sushigocrafting:sesame_seeds",
        ),
    ];
    let to: &str = rename.to.as_str();
    if !to.starts_with("sushigocrafting:") || !to.ends_with("_seeds") {
        return;
    }
    let Some(pair) = PAIRS.iter().find(|p| p.1 == to) else {
        return;
    };
    if let Some(block) = row.at("fields/block") {
        if matches!(
            block.as_str(),
            Some("minecraft:wheat") | Some("minecraft:beetroots")
        ) {
            *block = J::Str(pair.0.to_string());
        }
    }
}
