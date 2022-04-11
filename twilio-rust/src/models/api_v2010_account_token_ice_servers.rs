/*
 * Twilio - Api
 *
 * This is the public Twilio REST API.
 *
 * The version of the OpenAPI document: 1.28.1
 * Contact: support@twilio.com
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct ApiV2010AccountTokenIceServers {
    #[serde(rename = "credential", skip_serializing_if = "Option::is_none")]
    pub credential: Option<String>,
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "urls", skip_serializing_if = "Option::is_none")]
    pub urls: Option<String>,
    #[serde(rename = "username", skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

impl ApiV2010AccountTokenIceServers {
    pub fn new() -> ApiV2010AccountTokenIceServers {
        ApiV2010AccountTokenIceServers {
            credential: None,
            url: None,
            urls: None,
            username: None,
        }
    }
}


