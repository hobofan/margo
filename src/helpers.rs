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
