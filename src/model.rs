use url_serde;
use url::Url;
use Map;

#[derive(Deserialize, Debug)]
pub enum RequestType {
    Create,
    Update,
    Delete,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Status {
    Success,
    Failed,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CloudFormationRequest {
    pub request_type: RequestType,
    #[serde(rename = "ResponseURL", with = "url_serde")]
    pub response_url: Url,
    pub stack_id: String,
    pub request_id: String,
    pub resource_type: String,
    pub logical_resource_id: String,
    #[serde(default)]
    pub physical_resource_id: Option<String>,
    pub resource_properties: Map,
    #[serde(default)]
    pub old_resource_properties: Map,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CloudFormationResponse {
    pub status: Status,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub physical_resource_id: String,
    pub stack_id: String,
    pub request_id: String,
    pub logical_resource_id: String,
    pub data: Map,
}
