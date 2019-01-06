#[cfg(feature = "source_resolver_remote_registry")]
mod remote_registry;
#[cfg(feature = "source_resolver_remote_registry")]
pub use self::remote_registry::RemoteRegistrySourceResolver;

use futures::future::Future as StdFuture;

use crate::Crate;

pub trait SourceResolver<T> {
    type F: StdFuture<Item = T, Error = ()> + Send;

    fn resolve_crate(&self, _crate: &Crate) -> Self::F;
}
