use log::{debug, trace};
use sha2::{Digest, Sha256};
use std::path::Path;

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
        let mut hasher = Sha256::new();

        debug!("DL URL: {:?}", download_url);
        let mut dl_res = self
            .client
            .get(reqwest::Url::parse(&download_url).unwrap())
            .send()
            .unwrap();
        let mut buffer: Vec<u8> = vec![];
        dl_res.copy_to(&mut buffer).unwrap();
        debug!("FINISHED DL URL: {:?}", download_url);

        std::io::copy(&mut buffer.as_slice(), &mut hasher).unwrap();
        let calculated_checksum = hex::encode(hasher.result());
        trace!("CALCULATED CHECKSUM: {:?}", calculated_checksum);
        trace!("EXPECTED CHECKSUM: {:?}", checksum);
        let checksums_match = &calculated_checksum == checksum;
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
