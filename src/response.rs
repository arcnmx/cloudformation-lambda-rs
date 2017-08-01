use std::io::{self, Write};
use hyper::{status, method, header};
use hyper::client::Request;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use serde_json;
use url::Url;

use model::CloudFormationResponse;
use Error;

pub fn send(url: Url, response: CloudFormationResponse) -> Result<(), Error> {
    // Need to know Content-Length beforehand...
    let response_json = serde_json::to_string(&response)?;

    let tls = NativeTlsClient::new()?;
    let mut request = Request::with_connector(method::Method::Put, url, &HttpsConnector::new(tls))?;
    request.headers_mut().set(header::ContentLength(response_json.len() as _));
    let mut request = request.start()?;
    io::copy(&mut response_json.as_bytes(), &mut request)?;
    let mut response = request.send()?;

    if response.status != status::StatusCode::Ok {
        // Print out response body
        let stderr = io::stderr();
        {
            let mut stderr = stderr.lock();
            let _ = io::copy(&mut response, &mut stderr);
            let _ = stderr.flush();
        }

        error!("S3 response: {:?}", response);

        Err(Box::new(io::Error::new(io::ErrorKind::Other, format!("CloudFormation response failed with {}", response.status))))
    } else {
        Ok(())
    }
}

