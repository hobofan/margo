#[macro_use]
extern crate serde_derive;

mod helpers;
mod link_resolver;

use futures::future::Future;
use futures::stream::Stream;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::link_resolver::LinkResolver;

#[derive(Deserialize, Debug)]
pub struct CargoLockfile {
    metadata: HashMap<String, String>,
}

impl CargoLockfile {
    pub fn crates(&self) -> Vec<Crate> {
        self.metadata
            .iter()
            .map(|(crate_part, checksum)| Crate::from_parts(crate_part, checksum))
            .collect()
    }

    pub fn fetchable_crates(&self) -> Vec<Crate> {
        self.crates()
            .into_iter()
            .filter(|n| n.checksum.is_some())
            .filter(|n| n.source == "registry+https://github.com/rust-lang/crates.io-index")
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Crate {
    crate_name: String,
    version: String,
    source: String,
    checksum: Option<String>,
}

impl Crate {
    pub fn from_parts(crate_part: &str, checksum: &str) -> Self {
        let regex = Regex::new(r"(?m)checksum (.+) (.+) \((.+)\)").unwrap();
        let caps = regex.captures(crate_part).unwrap();

        let crate_name = caps.get(1).unwrap().as_str().to_owned();
        let version = caps.get(2).unwrap().as_str().to_owned();
        let source = caps.get(3).unwrap().as_str().to_owned();

        let checksum = match checksum {
            "<none>" => None,
            other => Some(other.to_owned()),
        };
        Self {
            crate_name,
            version,
            source,
            checksum,
        }
    }
}

pub trait CrateDownloadTarget {
    fn crate_name(&self) -> &str;
    fn version(&self) -> &str;
    fn checksum(&self) -> &str;

    fn target_path(&self) -> PathBuf;
}

#[derive(Debug)]
pub struct CargoCacheCrate<'a> {
    _crate: &'a Crate,
    cargo_dir: &'a Path,
    registry_name: &'a str,
}

impl<'a> CargoCacheCrate<'a> {
    pub fn new(_crate: &'a Crate, cargo_dir: &'a Path, registry_name: &'a str) -> Self {
        Self {
            _crate,
            cargo_dir,
            registry_name,
        }
    }

    pub fn cargo_crate(&self) -> &Crate {
        &self._crate
    }

    /// Path to where the cache file of the crate should be stored.
    pub fn cache_path(&self) -> PathBuf {
        let mut path = self
            .cargo_dir
            .join("registry")
            .join("cache")
            .join(self.registry_name);
        path = path.join(format!(
            "{}-{}.crate",
            self._crate.crate_name, self._crate.version
        ));

        path
    }
}

impl<'a> CrateDownloadTarget for CargoCacheCrate<'a> {
    fn crate_name(&self) -> &str {
        &self._crate.crate_name
    }

    fn version(&self) -> &str {
        &self._crate.version
    }

    fn checksum(&self) -> &str {
        self._crate.checksum.as_ref().unwrap()
    }

    fn target_path(&self) -> PathBuf {
        self.cache_path()
    }
}

fn main() {
    env_logger::init();

    let mut f = File::open("Cargo.lock").unwrap(); // TODO
    let mut buffer = vec![];
    f.read_to_end(&mut buffer).unwrap();

    let lockfile: CargoLockfile = toml::de::from_slice(&buffer).unwrap();

    println!("{:?}", lockfile.fetchable_crates());

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
        println!("{:?} : {:?}", cache_path, cache_path.exists());
    }

    println!(
        "Fetched: {} || Unfetched: {}",
        fetched_crates, unfetched_crates
    );

    let mut eloop = tokio_core::reactor::Core::new().unwrap();

    // TODO
    let remote_registry_url = "https://github.com/rust-lang/crates.io-index";
    let link_resolver = crate::link_resolver::RemoteRegistryLinkResolver::new(remote_registry_url);

    let client = Arc::new(reqwest::Client::new());
    let fetch_stream = futures::stream::futures_ordered(
        crate_download_targets.into_iter().map(futures::future::ok),
    )
    .filter(|cargo_crate| {
        println!("ABC: {:?}", !cargo_crate.target_path().exists());
        !cargo_crate.target_path().exists()
    })
    .and_then(|fetchable_crate| {
        println!("THERE");
        let client2 = client.clone();
        LinkResolver::resolve_crate(&link_resolver, &fetchable_crate.cargo_crate()).and_then(
            move |crate_dl_url| {
                let mut dl_dest = File::create(fetchable_crate.target_path()).unwrap();

                println!("DL URL: {:?}", crate_dl_url);
                client2
                    .get(reqwest::Url::parse(&crate_dl_url).unwrap())
                    .send()
                    .unwrap()
                    .copy_to(&mut dl_dest)
                    .unwrap();
                println!("FINISHED DL URL: {:?}", crate_dl_url);

                Ok(())
            },
        )
    })
    .collect();
    // .collect();
    // let fetch_stream = fetch_futures);

    eloop
        .run(fetch_stream.and_then(|_| Ok(())).map_err(|_| ()))
        .unwrap();
}
