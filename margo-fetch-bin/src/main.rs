use futures::future::Future;
use futures::stream::Stream;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use margo_fetch::downloader::Downloader;
use margo_fetch::source_resolver::SourceResolver;
use margo_fetch::CargoCacheCrate;
use margo_fetch::CargoLockfile;
use margo_fetch::CrateDownloadTarget;

fn main() {
    env_logger::init();

    let mut f = File::open("Cargo.lock").unwrap(); // TODO
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();

    let lockfile: CargoLockfile = toml::de::from_slice(&buffer).unwrap();

    let cargo_dir = Path::new("/Users/hobofan/.cargo/");
    // HACK: fixed for now, assuming that it will stay the same
    let registry_name = "github.com-1ecc6299db9ec823";
    let fetchable_crates: Vec<_> = lockfile.fetchable_crates();
    let crate_download_targets: Vec<_> = fetchable_crates
        .iter()
        .map(|n| CargoCacheCrate::new(&n, &cargo_dir, registry_name))
        .collect();

    let mut fetched_crates = 0;
    let mut unfetched_crates = 0;
    for fetchable_crate in &crate_download_targets {
        let cache_path = fetchable_crate.target_path();
        if cache_path.exists() {
            fetched_crates += 1;
        } else {
            unfetched_crates += 1;
        }
    }

    println!(
        "Fetched: {} || Unfetched: {}",
        fetched_crates, unfetched_crates
    );

    let mut eloop = tokio_core::reactor::Core::new().unwrap();

    // TODO
    let remote_registry_url = "https://github.com/rust-lang/crates.io-index";
    let source_resolver =
        margo_fetch::source_resolver::RemoteRegistrySourceResolver::new(remote_registry_url);

    let fetch_stream = futures::stream::futures_ordered(
        crate_download_targets.into_iter().map(futures::future::ok),
    )
    .filter(|cargo_crate| !cargo_crate.target_path().exists())
    .and_then(|fetchable_crate| {
        SourceResolver::resolve_crate(&source_resolver, &fetchable_crate.cargo_crate()).and_then(
            move |crate_dl_url| {
                let downloader = margo_fetch::downloader::SimpleReqwestDownloader::new();
                downloader.checked_download(
                    &crate_dl_url,
                    &fetchable_crate.target_path(),
                    fetchable_crate.cargo_crate().checksum.as_ref().unwrap(),
                )
            },
        )
    })
    .collect();

    eloop
        .run(fetch_stream.and_then(|_| Ok(())).map_err(|_| ()))
        .unwrap();
}
