#[macro_use]
extern crate log;
#[cfg(feature = "env_logger")]
extern crate env_logger;
extern crate unreachable;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
#[macro_use]
extern crate serde;
extern crate crowbar;
extern crate hyper;
extern crate hyper_native_tls;
extern crate url;
extern crate url_serde;

pub use crowbar::{Value, Context};
use serde::{Deserialize};
use url::Url;

pub type Error = Box<::std::error::Error>;

mod map;
mod response;
pub mod model;
pub use map::Map;

const PHYSICAL_RESOURCE_ID_FAILURE: &'static str = "FAILURE";
const SERVICE_TOKEN_KEY: &'static str = "ServiceToken";

#[derive(Clone, Debug)]
pub struct CloudFormationRequest<'a> {
    pub stack_id: &'a str,
    pub request_id: &'a str,
    pub resource_type: String,
    pub logical_resource_id: &'a str,
    pub resource_properties: Map,
}

#[derive(Clone, Default, Debug)]
pub struct CloudFormationResponse {
    pub physical_resource_id: String,
    pub data: Map,
}

impl CloudFormationResponse {
    pub fn empty(physical_resource_id: String) -> Self {
        CloudFormationResponse {
            physical_resource_id: physical_resource_id,
            data: Default::default(),
        }
    }
}

pub trait CloudFormationResource {
    type Error: ToString;

    fn create(self, context: Context, request: CloudFormationRequest) -> Result<CloudFormationResponse, Self::Error>;
    fn delete(self, context: Context, request: CloudFormationRequest, physical_resource_id: &str) -> Result<(), Self::Error>;
    fn update(self, context: Context, request: CloudFormationRequest, physical_resource_id: String, old_resource_properties: Map) -> Result<CloudFormationResponse, Self::Error>;
}

pub fn decode_event(event: Value) -> Result<model::CloudFormationRequest, serde_json::Error> {
    model::CloudFormationRequest::deserialize(event)
}

#[cfg(feature = "env_logger")]
#[inline]
pub fn logger_init() {
    if let Err(err) = env_logger::init() {
        println!("env_logger failed: {:?}", err);
    }
}

#[cfg(not(any(feature = "env_logger")))]
#[inline]
pub fn logger_init() { }

pub fn handle<T: CloudFormationResource>(resource: T, request: model::CloudFormationRequest, context: Context) -> Result<(), Error> {
    trace!("Request {:?}", request);
    let (response, url) = provision(resource, request, context);
    trace!("Response {:?}", response);
    response::send(url, response)
}

pub fn unhandled(request: model::CloudFormationRequest, _context: Context) -> Result<(), Error> {
    warn!("Unhandled request {:?}", request);

    let response = match (request.request_type, request.physical_resource_id) {
        (model::RequestType::Delete, ref mut physical_resource_id) if physical_resource_id.as_ref().map(|s| &s[..]) == Some(PHYSICAL_RESOURCE_ID_FAILURE) => model::CloudFormationResponse {
            status: model::Status::Success,
            reason: None,
            physical_resource_id: physical_resource_id.take().unwrap(),
            stack_id: request.stack_id,
            request_id: request.request_id,
            logical_resource_id: request.logical_resource_id,
            data: Default::default(),
        },
        (_, physical_resource_id) => model::CloudFormationResponse {
            status: model::Status::Failed,
            reason: Some(format!("Unknown ResourceType {}", request.resource_type)),
            physical_resource_id: physical_resource_id.unwrap_or_else(|| PHYSICAL_RESOURCE_ID_FAILURE.into()),
            stack_id: request.stack_id,
            request_id: request.request_id,
            logical_resource_id: request.logical_resource_id,
            data: Default::default(),
        },
    };
    response::send(request.response_url, response)
}

fn provision<T: CloudFormationResource>(resource: T, mut request: model::CloudFormationRequest, context: Context) -> (model::CloudFormationResponse, Url) {
    let res = {
        // Not necessary and can conflict with serde(deny_unknown_fields)
        request.resource_properties.remove(SERVICE_TOKEN_KEY);
        request.old_resource_properties.remove(SERVICE_TOKEN_KEY);

        let req = CloudFormationRequest {
            stack_id: &request.stack_id,
            request_id: &request.request_id,
            resource_type: request.resource_type,
            logical_resource_id: &request.logical_resource_id,
            resource_properties: request.resource_properties,
        };

        match (request.request_type, request.physical_resource_id) {
            (model::RequestType::Delete, ref mut physical_resource_id) if physical_resource_id.as_ref().map(|s| &s[..]) == Some(PHYSICAL_RESOURCE_ID_FAILURE) =>
                Ok(CloudFormationResponse::empty(physical_resource_id.take().unwrap())),
            (model::RequestType::Create, None) =>
                resource.create(context, req).map_err(|e| e.to_string()).map_err(|e| (e, PHYSICAL_RESOURCE_ID_FAILURE.into())),
            (model::RequestType::Update, Some(physical_resource_id)) =>
                resource.update(context, req, physical_resource_id.clone(), request.old_resource_properties).map_err(|e| (e.to_string(), physical_resource_id)),
            (model::RequestType::Delete, Some(physical_resource_id)) =>
                match resource.delete(context, req, &physical_resource_id) {
                    Ok(_) => Ok(CloudFormationResponse::empty(physical_resource_id)),
                    Err(e) => Err((e.to_string(), physical_resource_id)),
                },
            (model::RequestType::Create, Some(physical_resource_id)) => Err((format!("Unexpected physical_resource_id {}", physical_resource_id), physical_resource_id)),
            (_, None) => Err(("physical_resource_id expected".into(), PHYSICAL_RESOURCE_ID_FAILURE.into())),
        }
    };

    (match res {
        Ok(res) => model::CloudFormationResponse {
            status: model::Status::Success,
            reason: None,
            physical_resource_id: res.physical_resource_id,
            stack_id: request.stack_id,
            request_id: request.request_id,
            logical_resource_id: request.logical_resource_id,
            data: res.data,
        },
        Err((err, physical_resource_id)) => model::CloudFormationResponse {
            status: model::Status::Failed,
            reason: Some(err.to_string()),
            physical_resource_id: physical_resource_id,
            stack_id: request.stack_id,
            request_id: request.request_id,
            logical_resource_id: request.logical_resource_id,
            data: Default::default(),
        },
    }, request.response_url)
}

#[macro_export]
macro_rules! cloudformation {
    ($($name:expr => $target:expr,)*) => {
        (|event: $crate::Value, context: $crate::Context| -> Result<(), $crate::Error> {
            $crate::logger_init();
            let request = $crate::decode_event(event)?;
            match &request.resource_type[..] {
            $(
                $name => $crate::handle($target, request, context),
            )*
                _ => $crate::unhandled(request, context),
            }
        })
    };
}
