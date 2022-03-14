/// All requests and response types used to communicate with devices in the Intercom.
use crate::device::terminal::{Terminal, Text};
use crate::request::*;
use serde::{Deserialize, Serialize};
use std::net::TcpStream;

pub struct ThreadRequest(pub Requests, pub TcpStream);

#[derive(Serialize, Deserialize)]
pub enum Requests {
    TerminalGetText(BasicGetRequest<Terminal, Text>),
    TerminalSetText(BasicSetRequest<Terminal, Text>),
}

#[derive(Serialize, Deserialize, Clone)]
pub enum Responses {
    TerminalGetText(BasicGetResponse<Terminal, Text>),
    TerminalSetText(BasicSetResponse<Terminal, Text>),
}
