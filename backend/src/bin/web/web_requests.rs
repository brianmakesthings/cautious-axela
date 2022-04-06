use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeviceCommand(pub Commands);
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
#[derive(Deserialize, Debug, Clone)]
pub struct WebSocketRequest {
    pub id: String,
    pub command: Commands,
    pub message: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Commands {
    RtcSession,
    DoorGet,
    DoorSet,
    Ping,
    TerminalGet,
    TerminalSet,
    Unknown
}
