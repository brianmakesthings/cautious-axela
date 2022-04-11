# ApiV2010AccountSipSipIpAccessControlListSipIpAddress

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The unique id of the Account that is responsible for this resource. | [optional]
**cidr_prefix_length** | Option<**i32**> | An integer representing the length of the CIDR prefix to use with this IP address when accepting traffic. By default the entire IP address is used. | [optional]
**date_created** | Option<**String**> | The date that this resource was created, given as GMT in RFC 2822 format. | [optional]
**date_updated** | Option<**String**> | The date that this resource was last updated, given as GMT in RFC 2822 format. | [optional]
**friendly_name** | Option<**String**> | A human readable descriptive text for this resource, up to 64 characters long. | [optional]
**ip_access_control_list_sid** | Option<**String**> | The unique id of the IpAccessControlList resource that includes this resource. | [optional]
**ip_address** | Option<**String**> | An IP address in dotted decimal notation from which you want to accept traffic. Any SIP requests from this IP address will be allowed by Twilio. IPv4 only supported today. | [optional]
**sid** | Option<**String**> | A 34 character string that uniquely identifies this resource. | [optional]
**uri** | Option<**String**> | The URI for this resource, relative to https://api.twilio.com | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


