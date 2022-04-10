/// All requests and response types used to communicate with devices in the Intercom.
use crate::device::door::{Door, DoorState};
use crate::device::keypad::{Code, KeyPad};
use crate::device::terminal::{Terminal, Text};
use crate::device::nfc::{NFCdev, NFCids};
use crate::request::*;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;

pub struct ThreadRequest(pub Requests, pub TcpStream);
pub struct InternalThreadRequest(pub Requests);

#[derive(Serialize, Deserialize)]
pub enum Requests {
    TerminalGetText(BasicGetRequest<Terminal, Text>),
    TerminalSetText(BasicSetRequest<Terminal, Text>),
    NFCGetID(BasicGetRequest<NFCdev, NFCids>),
    NFCSetID(BasicSetRequest<NFCdev, NFCids>),
    DoorGetState(BasicGetRequest<Door, DoorState>),
    DoorSetState(BasicSetRequest<Door, DoorState>),
    KeyPadGetCode(BasicGetRequest<KeyPad, Code>),
    KeyPadSetCode(BasicSetRequest<KeyPad, Code>),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Responses {
    TerminalGetText(BasicGetResponse<Terminal, Text>),
    TerminalSetText(BasicSetResponse<Terminal, Text>),
    NFCGetID(BasicGetResponse<NFCdev, NFCids>),
    NFCSetID(BasicSetResponse<NFCdev, NFCids>),
    DoorGetState(BasicGetResponse<Door, DoorState>),
    DoorSetState(BasicSetResponse<Door, DoorState>),
    KeyPadGetCode(BasicGetResponse<KeyPad, Code>),
    KeyPadSetCode(BasicSetResponse<KeyPad, Code>),
}
