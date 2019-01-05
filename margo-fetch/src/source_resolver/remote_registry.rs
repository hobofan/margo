use futures::future::Future as StdFuture;
use hyper::{client::HttpConnector, Client};
use hyper_tls::HttpsConnector;
use std::sync::Arc;
use std::sync::RwLock;

use self::helpers::follow_get;
use super::{Error, Future, SourceResolver};
use crate::Crate;

#[derive(Clone)]
pub struct RemoteRegistrySourceResolver {
    remote_registry_url: String,
    client: Arc<Client<HttpsConnector<HttpConnector>, hyper::Body>>,
    cached_registry_dl_url: Arc<RwLock<Option<String>>>,
}

impl RemoteRegistrySourceResolver {
    pub fn new(remote_registry_url: &str) -> Self {
        let https = HttpsConnector::new(4).expect("TLS initialization failed");
        let client = Arc::new(Client::builder().build::<_, hyper::Body>(https));

        Self {
            remote_registry_url: remote_registry_url.to_owned(),
            client,
            cached_registry_dl_url: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get_registry_dl_url(&self) -> impl StdFuture<Item = String> + Send {
        // TODO: adjust to non-Github-opinionated version
        let registry_config = format!("{}/raw/master/config.json", self.remote_registry_url);
        let registry_config_url = registry_config.parse().unwrap();

        let client2 = self.client.clone();
        let cached_registry_dl_url = self.cached_registry_dl_url.clone();
        futures::future::lazy(move || {
            let is_cached = { cached_registry_dl_url.read().unwrap().is_some() };
            match is_cached {
                false => futures::future::Either::A(
                    follow_get(&client2, registry_config_url).and_then(move |body: String| {
                        let content: serde_json::Value = serde_json::from_str(&body).unwrap();
                        let registry_dl_url: String =
                            content.get("dl").unwrap().as_str().unwrap().to_owned();

                        {
                            let mut cached_registry_dl_url =
                                cached_registry_dl_url.write().unwrap();
                            *cached_registry_dl_url = Some(registry_dl_url.clone());
                        }

                        futures::future::ok(registry_dl_url)
                    }),
                ),
                true => {
                    let registry_dl_url: String = cached_registry_dl_url
                        .read()
                        .unwrap()
                        .as_ref()
                        .unwrap()
                        .to_owned();
                    futures::future::Either::B(futures::future::ok(registry_dl_url))
                }
            }
        })
    }

    pub fn resolve_crate_download_link(
        &self,
        _crate: &Crate,
        registry_dl_url: String,
    ) -> Future<String> {
        Box::new(futures::future::ok(format!(
            "{}/{}/{}/download",
            registry_dl_url, _crate.crate_name, _crate.version
        )))
    }

    pub fn resolve_crate(
        &self,
        _crate: Crate,
    ) -> impl StdFuture<Item = String, Error = Error> + Send {
        let resolver = self.clone();
        futures::future::lazy(|| {
            let registry_dl_url = resolver.get_registry_dl_url().map_err(|_| ());

            registry_dl_url
                .and_then(move |registry_dl_url| {
                    resolver.resolve_crate_download_link(&_crate, registry_dl_url)
                })
                .map_err(|_| ())
        })
    }
}

#[must_use = "futures do nothing unless polled"]
pub struct RemoteRegistrySourceResolverFuture {
    inner: Future<String>,
}

impl RemoteRegistrySourceResolverFuture {
    fn new(fut: Future<String>) -> Self {
        Self { inner: fut }
    }
}

impl StdFuture for RemoteRegistrySourceResolverFuture {
    type Item = String;
    type Error = ();

    fn poll(&mut self) -> futures::Poll<Self::Item, Self::Error> {
        self.inner.poll()
    }
}

impl SourceResolver for RemoteRegistrySourceResolver {
    type F = RemoteRegistrySourceResolverFuture;

    fn resolve_crate(&self, _crate: &Crate) -> Self::F {
        RemoteRegistrySourceResolverFuture::new(Box::new(self.resolve_crate(_crate.clone())))
    }
}

mod helpers {
    use futures::future::Either;
    use futures::future::Future;
    use futures::stream::Stream;
    use hyper::body::Payload;
    use hyper::client::connect::Connect;
    use hyper::Client;
    use hyper::Uri;

    pub fn follow_get<C, B>(
        client: &Client<C, B>,
        url: Uri,
    ) -> impl Future<Item = String, Error = hyper::Error> + Send
    where
        C: Connect + Sync + 'static,
        C::Transport: 'static,
        C::Future: 'static,
        B: Default + Payload + Send + 'static,
        B::Data: Send,
    {
        let second_client = client.clone();

        client
            .get(url)
            .and_then(move |res| {
                let location_header = res.headers().get("Location").unwrap();
                let redirect_url = location_header.to_str().unwrap().parse().unwrap();
                match res.status().is_redirection() {
                    true => Either::A(second_client.get(redirect_url)),
                    false => Either::B(futures::future::ok(res)),
                }
            })
            .and_then(|res| {
                res.into_body().concat2().and_then(|body| {
                    Ok(String::from_utf8(body.into_bytes().as_ref().to_owned()).unwrap())
                })
            })
    }
}
