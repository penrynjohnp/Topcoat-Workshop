use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use slipway::assets::SLIPWAY_MARK;
use topcoat::asset::AssetConfig;
use topcoat_asset::{Bundler, BundlerConfig};

// ANCHOR: hashed-asset-test
#[test]
fn local_asset_resolves_to_a_content_hashed_url() {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("lab12-assets-{suffix}"));
    let bundle_dir = root.join("assets");
    let cache_dir = root.join("cache");
    let binary = fs::read(std::env::current_exe().unwrap()).unwrap();

    Bundler::new(&BundlerConfig::new().cache_dir(cache_dir))
        .bundle(&binary, &bundle_dir)
        .unwrap();
    let bundle = topcoat::asset::AssetBundle::load_dir(&bundle_dir).unwrap();
    let url = AssetConfig::hosted_at("/_topcoat/assets", bundle).resolve(SLIPWAY_MARK);

    let file = url.strip_prefix("/_topcoat/assets/slipway-mark-").unwrap();
    let hash = file.strip_suffix(".svg").unwrap();
    assert_eq!(hash.len(), 16, "{url}");
    assert!(hash.bytes().all(|byte| byte.is_ascii_hexdigit()), "{url}");

    fs::remove_dir_all(root).unwrap();
}
// ANCHOR_END: hashed-asset-test
