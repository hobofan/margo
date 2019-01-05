use log::{debug, trace};
use std::path::Path;

use super::checksum::verify_checksum;
use super::{Downloader, Future};

pub struct SimpleReqwestDownloader {
    client: reqwest::Client,
}

impl SimpleReqwestDownloader {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Downloader for SimpleReqwestDownloader {
    type F = Future<()>;

    fn checked_download(&self, download_url: &str, target_path: &Path, checksum: &str) -> Self::F {
        debug!("DL URL: {:?}", download_url);
        let mut dl_res = self
            .client
            .get(reqwest::Url::parse(&download_url).unwrap())
            .send()
            .unwrap();
        let mut buffer: Vec<u8> = vec![];
        dl_res.copy_to(&mut buffer).unwrap();
        debug!("FINISHED DL URL: {:?}", download_url);

        let checksums_match = verify_checksum(&mut buffer.as_slice(), checksum);
        trace!("CHECKSUM matches: {:?}", checksums_match);
        if !checksums_match {
            return Box::new(futures::future::err(()));
        }

        let mut dl_dest = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(target_path)
            .unwrap();
        std::io::copy(&mut buffer.as_slice(), &mut dl_dest).unwrap();

        Box::new(futures::future::ok(()))
    }
}
