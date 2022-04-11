use crate::web_requests::*;
use common::device::door::{Door, DoorState};
use common::device::keypad::{Code, KeyPad, PhoneNumberText};
use common::device::nfc::{NFCdev, NFCids};
use common::device::terminal::{Terminal, Text};
use common::message::{read_from_stream, write_to_stream};
use common::request::*;
use common::requests_and_responses::{Requests, Responses};
use std::env;
use std::marker::PhantomData;
use std::net::TcpStream;
use std::process;

static mut INTERCOM_ID: IDManage = IDManage { id: 0 };

#[derive(Clone, Copy)]
struct IDManage {
    pub id: u128,
}

impl IDManage {
    fn get_id(&mut self) -> u128 {
        let id_num = self.id;
        self.id += 1;
        id_num
    }
}

impl Commands {
    fn set_command(&self, request: WebRequests) -> (Requests, u128) {
        let msg = request.get_msg().0;
        let id = unsafe { INTERCOM_ID.get_id() };

        match self {
            Commands::TerminalGet => (
                Requests::TerminalGetText(BasicGetRequest::<Terminal, Text>(
                    ID(id),
                    PhantomData,
                    PhantomData,
                )),
                id,
            ),
            Commands::TerminalSet => (
                Requests::TerminalSetText(BasicSetRequest::<Terminal, Text>(
                    ID(id),
                    Text(msg.to_string()),
                    PhantomData,
                )),
                id,
            ),
            Commands::DoorGet => (
                Requests::DoorGetState(BasicGetRequest::<Door, DoorState>(
                    ID(id),
                    PhantomData,
                    PhantomData,
                )),
                id,
            ),
            Commands::DoorSet => {
                let door_state = serde_json::from_str(&msg).unwrap();
                (
                    Requests::DoorSetState(BasicSetRequest::<Door, DoorState>(
                        ID(id),
                        door_state,
                        PhantomData,
                    )),
                    id,
                )
            }
            Commands::NFCGet => (
                Requests::NFCGetID(BasicGetRequest::<NFCdev, NFCids>(
                    ID(id),
                    PhantomData,
                    PhantomData,
                )),
                id,
            ),
            Commands::KeypadGetCode => (
                Requests::KeyPadGetCode(BasicGetRequest::<KeyPad, Code>(
                    ID(id),
                    PhantomData,
                    PhantomData,
                )),
                id,
            ),
            Commands::PhoneGet => (
                Requests::PhoneGet(BasicGetRequest::<KeyPad, PhoneNumberText>(
                    ID(id),
                    PhantomData,
                    PhantomData,
                )),
                id,
            ),
            Commands::NFCSet => (
                Requests::NFCSetID(BasicSetRequest::<NFCdev, NFCids>(
                    ID(id),
                    NFCids(msg.to_string()),
                    PhantomData,
                )),
                id,
            ),
            Commands::KeypadSetCode => {
                let code = serde_json::from_str(&msg).unwrap();
                (
                    Requests::KeyPadSetCode(BasicSetRequest::<KeyPad, Code>(
                        ID(id),
                        code,
                        PhantomData,
                    )),
                    id,
                )
            }
            Commands::PhoneSet => {
                let phone_number = serde_json::from_str(&msg).unwrap();
                (
                    Requests::PhoneSet(BasicSetRequest::<KeyPad, PhoneNumberText>(
                        ID(id),
                        phone_number,
                        PhantomData,
                    )),
                    id,
                )
            }
            _ => (
                Requests::TerminalSetText(BasicSetRequest::<Terminal, Text>(
                    ID(id),
                    Text(msg.to_string()),
                    PhantomData,
                )),
                id,
            ),
        }
    }
}

pub fn match_intercom_response(response: Responses, id: u128) -> String {
    let mut message = String::from("");
    match response {
        Responses::TerminalGetText(msg_get) => {
            assert_eq!(msg_get.get_id().0, id);
            message = msg_get.get_result().unwrap().0;
        }
        Responses::TerminalSetText(msg_set) => {
            assert_eq!(msg_set.get_id().0, id);
            let msg = msg_set.get_candidate().clone();
            message = msg.0;
        }
        Responses::DoorGetState(msg_get) => {
            assert_eq!(msg_get.get_id().0, id);
            let msg = msg_get.get_result().unwrap();
            message = serde_json::to_string(&msg).unwrap();
        }
        Responses::DoorSetState(msg_set) => {
            assert_eq!(msg_set.get_id().0, id);
            let msg = msg_set.get_candidate().clone();
            message = serde_json::to_string(&msg).unwrap();
        }
        Responses::KeyPadGetCode(msg_get) => {
            assert_eq!(msg_get.get_id().0, id);
            let msg = msg_get.get_result().unwrap();
            message = serde_json::to_string(&msg).unwrap();
        }
        Responses::KeyPadSetCode(msg_set) => {
            assert_eq!(msg_set.get_id().0, id);
            let msg = msg_set.get_candidate().clone();
            message = serde_json::to_string(&msg).unwrap();
        }
        Responses::PhoneGet(msg_get) => {
            assert_eq!(msg_get.get_id().0, id);
            let msg = msg_get.get_result().unwrap();
            message = serde_json::to_string(&msg).unwrap();
        }
        Responses::PhoneSet(msg_set) => {
            assert_eq!(msg_set.get_id().0, id);
            let msg = msg_set.get_candidate().clone();
            message = serde_json::to_string(&msg).unwrap();
        }
        Responses::NFCGetID(_) => {
            if let Responses::NFCGetID(msg_get) = response {
                assert_eq!(msg_get.get_id().0, id);
                let msg = msg_get.get_result().unwrap();
                message = serde_json::to_string(&msg).unwrap();
            }
        }
        Responses::NFCSetID(_) => {
            if let Responses::NFCSetID(msg_set) = response {
                assert_eq!(msg_set.get_id().0, id);
                let msg = msg_set.get_candidate().clone();
                message = msg.0;
            }
        }
    }
    message
}

// call from client_msg() in main
// relays message from web to intercom
pub async fn listen_for_web(request: WebSocketRequest) -> String {
    let new_request = WebRequests(DeviceCommand(request.command), Message(request.message));

    let request_command = new_request.clone();
    // let command = Commands::match_command(new_request.get_device_command().0);
    let command = request_command.get_device_command().0;
    let (setrequest, id): (Requests, u128) = Commands::set_command(&command, request_command);

    // send command to intercom and get reply
    let response = send_command_to_intercom(setrequest).await;
    let reply_to_web = match_intercom_response(response, id);

    reply_to_web
}

async fn send_command_to_intercom(request: Requests) -> Responses {
    let message = tokio::task::spawn(async move {
        match TcpStream::connect(format!("{}:2000", env::var("INTERCOM_ADDRESS").unwrap())) {
            Ok(mut intercom_stream) => {
                // send command to intercom
                write_to_stream(&mut intercom_stream, &request);

                // get reply
                let reply: Responses = read_from_stream(&mut intercom_stream).unwrap();
                reply
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
                process::exit(1);
            }
        }
    })
    .await
    .unwrap();

    message
}
