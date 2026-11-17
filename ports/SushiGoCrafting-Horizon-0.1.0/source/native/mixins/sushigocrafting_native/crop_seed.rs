// The crop's gameplay row names its seeds as its item (creative pick block), as a crop's own
// seeds are named in vanilla. Self-contained: this runs in `mod_content.rs`, which the game
// builds into `horizon_client_lib` (no `crate::game` there). Same table as `seed_crop.rs`.
fn sushigocrafting_crop_seed(block: &NewBlock, row: &mut J) {
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
    let id: &str = block.decl.id.as_str();
    if !id.starts_with("sushigocrafting:") {
        return;
    }
    if let Some(pair) = PAIRS.iter().find(|p| p.0 == id) {
        row.set("item", J::Str(pair.1.to_string()));
    }
}
