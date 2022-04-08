/// All requests and response types used to communicate with devices in the Intercom.
use crate::device::door::{Door, DoorState};
use crate::device::terminal::{Terminal, Text};
use crate::device::nfc::{NFC, NFCids};
use crate::request::*;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;

pub struct ThreadRequest(pub Requests, pub TcpStream);

#[derive(Serialize, Deserialize)]
pub enum Requests {
    TerminalGetText(BasicGetRequest<Terminal, Text>),
    TerminalSetText(BasicSetRequest<Terminal, Text>),
    NFCGetID(BasicGetRequest<NFC, NFCids>),
    NFCSetID(BasicSetRequest<NFC, NFCids>),
    DoorGetState(BasicGetRequest<Door, DoorState>),
    DoorSetState(BasicSetRequest<Door, DoorState>),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Responses {
    TerminalGetText(BasicGetResponse<Terminal, Text>),
    TerminalSetText(BasicSetResponse<Terminal, Text>),
    NFCGetID(BasicGetResponse<NFC, NFCids>),
    NFCSetID(BasicSetResponse<NFC, NFCids>),
    DoorGetState(BasicGetResponse<Door, DoorState>),
    DoorSetState(BasicSetResponse<Door, DoorState>),
}
