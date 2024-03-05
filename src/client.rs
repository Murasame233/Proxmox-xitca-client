use std::str::FromStr;

use http::{uri::Builder as UriBuilder, HeaderValue, Uri};
use serde::{Deserialize, Serialize};
use xitca_client::{middleware::Decompress, Client, RequestBuilder};

use xitca_client::{error::Error, http::Version, Io, Service};

use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};

#[derive(Serialize, Deserialize, Debug)]
pub struct ProxmoxRes<T> {
    pub data: T,
}

pub fn get_uri_builder(base: &'_ str) -> UriBuilder {
    let uri_base = Uri::from_str(base).unwrap();
    let auth = uri_base.authority().unwrap();
    let scheme = uri_base.scheme().unwrap();
    Uri::builder()
        .authority(auth.as_str())
        .scheme(scheme.as_str())
}

pub struct ProxmoxApiService {
    client: Client,
    base: Uri,
    auth: HeaderValue,
}

impl ProxmoxApiService {
    pub async fn new(base: &'_ str, auth: &str) -> Self {
        let client = Client::builder()
            .middleware(Decompress::new)
            .tls_connector(MySSLConnector::new())
            .finish();
        let base = get_uri_builder(base).path_and_query("/").build().unwrap();

        let auth = HeaderValue::from_str(auth).unwrap();
        Self { client, base, auth }
    }

    pub async fn make_get_request(&self, path: &'_ str) -> RequestBuilder<'_> {
        let uri = UriBuilder::from(self.base.clone())
            .path_and_query(path)
            .build()
            .unwrap();
        let mut req = self.client.get(uri).unwrap();
        self.add_auth(&mut req);
        req
    }
    pub async fn make_post_request<T>(&self, path: &'_ str, data: T) -> RequestBuilder<'_>
    where
        T: Serialize,
    {
        let uri = self.get_uri_builder().path_and_query(path).build().unwrap();
        let mut bytes: Vec<u8> = Vec::new();
        serde_json::to_writer(&mut bytes, &data).unwrap();
        let mut req = self.client.get(uri).unwrap();
        self.add_auth(&mut req);
        req.body(bytes)
    }

    fn add_auth(&self, req: &mut RequestBuilder<'_>) {
        req.headers_mut().insert("Authorization", self.auth.clone());
    }
    fn get_uri_builder(&self) -> UriBuilder {
        UriBuilder::from(self.base.clone())
    }
}

impl<'n> Service<(&'n str, Box<dyn Io>)> for MySSLConnector {
    type Response = (Box<dyn Io>, Version);
    type Error = Error;

    async fn call(&self, req: (&'n str, Box<dyn Io>)) -> Result<Self::Response, Self::Error> {
        self.0.call(req).await
    }
}

struct MySSLConnector(SslConnector);

impl MySSLConnector {
    fn new() -> Self {
        let mut ssl = SslConnector::builder(SslMethod::tls()).unwrap();

        ssl.set_verify(SslVerifyMode::empty());

        ssl.set_alpn_protos(b"\x08http/1.1\x02h2").unwrap();

        MySSLConnector(ssl.build())
    }
}
