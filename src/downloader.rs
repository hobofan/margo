mod simple_reqwest_downloader;
pub use self::simple_reqwest_downloader::SimpleReqwestDownloader;

use futures::future::Future as StdFuture;
use std::path::Path;

type Error = ();

type Future<T> = Box<dyn StdFuture<Item = T, Error = Error> + Send>;

pub trait Downloader {
    type F: StdFuture<Item = (), Error = ()> + Send;

    fn checked_download(&self, download_url: &str, target_path: &Path, checksum: &str) -> Self::F;
}
