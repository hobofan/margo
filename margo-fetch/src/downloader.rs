#[cfg(feature = "downloader_simple_reqwest")]
mod simple_reqwest;
#[cfg(feature = "downloader_simple_reqwest")]
pub use self::simple_reqwest::SimpleReqwestDownloader;

use futures::future::Future as StdFuture;
use std::path::Path;

#[allow(dead_code)]
type Error = ();

#[allow(dead_code)]
#[cfg(feature = "std")]
type Future<T> = Box<dyn StdFuture<Item = T, Error = Error> + Send>;

// TODO: find an alternative to path, so it is also usable with no_std
#[cfg(feature = "std")]
pub trait Downloader {
    type F: StdFuture<Item = (), Error = ()> + Send;

    fn checked_download(&self, download_url: &str, target_path: &Path, checksum: &str) -> Self::F;
}

#[cfg(feature = "downloader_checksum")]
pub use self::checksum::verify_checksum;
#[cfg(feature = "downloader_checksum")]
pub mod checksum {
    use log::trace;
    use sha2::{Digest, Sha256};

    /// Helper to verify the checksum of a download.
    pub fn verify_checksum<R: std::io::Read>(input: &mut R, checksum: &str) -> bool {
        let mut hasher = Sha256::new();

        std::io::copy(input, &mut hasher).unwrap();
        let calculated_checksum = hex::encode(hasher.result());
        trace!("CALCULATED CHECKSUM: {:?}", calculated_checksum);
        trace!("EXPECTED CHECKSUM: {:?}", checksum);
        let checksums_match = &calculated_checksum == checksum;

        checksums_match
    }
}
