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
pub struct ApiV2010AccountMessageMessageFeedback {
    /// The SID of the Account that created the resource
    #[serde(rename = "account_sid", skip_serializing_if = "Option::is_none")]
    pub account_sid: Option<String>,
    /// The RFC 2822 date and time in GMT that the resource was created
    #[serde(rename = "date_created", skip_serializing_if = "Option::is_none")]
    pub date_created: Option<String>,
    /// The RFC 2822 date and time in GMT that the resource was last updated
    #[serde(rename = "date_updated", skip_serializing_if = "Option::is_none")]
    pub date_updated: Option<String>,
    /// The SID of the Message resource for which the feedback was provided
    #[serde(rename = "message_sid", skip_serializing_if = "Option::is_none")]
    pub message_sid: Option<String>,
    /// Whether the feedback has arrived
    #[serde(rename = "outcome", skip_serializing_if = "Option::is_none")]
    pub outcome: Option<Outcome>,
    /// The URI of the resource, relative to `https://api.twilio.com`
    #[serde(rename = "uri", skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
}

impl ApiV2010AccountMessageMessageFeedback {
    pub fn new() -> ApiV2010AccountMessageMessageFeedback {
        ApiV2010AccountMessageMessageFeedback {
            account_sid: None,
            date_created: None,
            date_updated: None,
            message_sid: None,
            outcome: None,
            uri: None,
        }
    }
}

/// Whether the feedback has arrived
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Outcome {
    #[serde(rename = "confirmed")]
    Confirmed,
    #[serde(rename = "unconfirmed")]
    Unconfirmed,
}

impl Default for Outcome {
    fn default() -> Outcome {
        Self::Confirmed
    }
}

