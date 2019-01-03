mod remote_registry;

pub use self::remote_registry::RemoteRegistryLinkResolver;

use futures::future::Future as StdFuture;

use crate::Crate;

type Error = ();

type Future<T> = Box<dyn StdFuture<Item = T, Error = Error> + Send>;

pub trait LinkResolver {
    type F: StdFuture<Item = String, Error = ()> + Send;

    fn resolve_crate(&self, _crate: &Crate) -> Self::F;
}
