#[cfg(feature = "source_resolver_remote_registry")]
mod remote_registry;
#[cfg(feature = "source_resolver_remote_registry")]
pub use self::remote_registry::RemoteRegistrySourceResolver;

use futures::future::Future as StdFuture;

use crate::Crate;

#[allow(dead_code)]
type Error = ();

#[allow(dead_code)]
type Future<T> = Box<dyn StdFuture<Item = T, Error = Error> + Send>;

pub trait SourceResolver {
    type F: StdFuture<Item = String, Error = ()> + Send;

    fn resolve_crate(&self, _crate: &Crate) -> Self::F;
}
