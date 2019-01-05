#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate serde_derive;

pub mod downloader;
mod helpers;
pub mod source_resolver;

#[cfg(feature = "std")]
use regex::Regex;
#[cfg(feature = "std")]
use std::collections::HashMap;
#[cfg(feature = "std")]
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
#[cfg(feature = "std")]
pub struct CargoLockfile {
    metadata: HashMap<String, String>,
}

impl CargoLockfile {
    #[cfg(feature = "std")]
    pub fn crates(&self) -> Vec<Crate> {
        self.metadata
            .iter()
            .map(|(crate_part, checksum)| Crate::parse_from_parts(crate_part, checksum))
            .collect()
    }

    #[cfg(feature = "std")]
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
    pub crate_name: String,
    pub version: String,
    pub source: String,
    pub checksum: Option<String>,
}

impl Crate {
    #[cfg(feature = "std")]
    pub fn parse_from_parts(crate_part: &str, checksum: &str) -> Self {
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

#[cfg(feature = "std")]
pub trait CrateDownloadTarget {
    fn crate_name(&self) -> &str;
    fn version(&self) -> &str;
    fn checksum(&self) -> &str;

    fn target_path(&self) -> PathBuf;
}

#[cfg(feature = "std")]
#[derive(Debug)]
pub struct CargoCacheCrate<'a> {
    _crate: &'a Crate,
    cargo_dir: &'a Path,
    registry_name: &'a str,
}

#[cfg(feature = "std")]
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

#[cfg(feature = "std")]
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
