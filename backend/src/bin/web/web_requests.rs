use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceCommand(pub String);
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message(pub String);
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebRequests(pub DeviceCommand, pub Message);

impl WebRequests {

  pub fn get_device_command(&self) -> DeviceCommand {
    let command = self.clone();
    return command.0
  }
  pub fn get_msg(&self) -> Message {
    let message = self.clone();
    return message.1
  }
}

// request from web
#[derive(Deserialize, Debug)]
pub struct WebSocketRequest {
    pub id: String,
    pub command: Commands,
    pub message: String,
}


#[derive(Deserialize, Debug)]
pub enum Commands {
    RtcSession,
    Lock,
    Ping,
    TerminalGet,
    TerminalSet,
    Unknown
}
