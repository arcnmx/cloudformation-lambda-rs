use std::io;
use hyper::{status, method};
use hyper::client::Request;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use serde::Serialize;
use serde_json;
use url::Url;

use model::CloudFormationResponse;
use Error;

pub fn send(url: Url, response: CloudFormationResponse) -> Result<(), Error> {
    let tls = NativeTlsClient::new()?;
    let mut request = Request::with_connector(method::Method::Put, url, &HttpsConnector::new(tls))?.start()?;
    response.serialize(&mut serde_json::Serializer::new(&mut request))?;
    let mut response = request.send()?;
    println!("Got response: {:#?}", response);

    if response.status != status::StatusCode::Ok {
        let _ = io::copy(&mut response, &mut io::stderr());
        Err(Box::new(io::Error::new(io::ErrorKind::Other, format!("CloudFormation response failed with {}", response.status))))
    } else {
        Ok(())
    }
}

