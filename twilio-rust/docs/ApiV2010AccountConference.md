# ApiV2010AccountConference

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the Account that created this resource | [optional]
**api_version** | Option<**String**> | The API version used to create this conference | [optional]
**call_sid_ending_conference** | Option<**String**> | The call SID that caused the conference to end | [optional]
**date_created** | Option<**String**> | The RFC 2822 date and time in GMT that this resource was created | [optional]
**date_updated** | Option<**String**> | The RFC 2822 date and time in GMT that this resource was last updated | [optional]
**friendly_name** | Option<**String**> | A string that you assigned to describe this conference room | [optional]
**reason_conference_ended** | Option<**String**> | The reason why a conference ended. | [optional]
**region** | Option<**String**> | A string that represents the Twilio Region where the conference was mixed | [optional]
**sid** | Option<**String**> | The unique string that identifies this resource | [optional]
**status** | Option<**String**> | The status of this conference | [optional]
**subresource_uris** | Option<[**serde_json::Value**](.md)> | A list of related resources identified by their relative URIs | [optional]
**uri** | Option<**String**> | The URI of this resource, relative to `https://api.twilio.com` | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


