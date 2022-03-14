use std::thread;
use std::net::{TcpListener, TcpStream};
use std::marker::PhantomData;
use std::process;
use std::sync::mpsc::Sender;
use common::message::{read_from_stream, write_to_stream};
use common::device::terminal::{Terminal, Text};
use common::requests_and_responses::{Requests, Responses};
use common::request::*;
use crate::web_requests::*;


static mut INTERCOM_ID: IDManage = IDManage{id:0};

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

// Type for the commands from web
enum Commands {
	TerminalGet,
	TerminalSet,
    // DoorGet,
	// DoorSet,
	// AudioGet,
	// AudioSet,
	// CameraGet,
	// CameraSet,
	Unknown
}


impl Commands {

	fn match_command(device: String) -> Commands {
		let device: &str = &*device;
		match device {
			"terminalget" => Commands::TerminalGet,
			"terminalset" => Commands::TerminalSet,
			// "doorget" => Commands::DoorGet,
			// "doorset" => Commands::DoorSet,
			// "audioget" => Commands::AudioGet,
            // "audioset" => Commands::AudioSet,
			// "cameraget" => Commands::CameraGet,
            // "cameraset" => Commands::CameraSet,
			_=> Commands::Unknown,
		}
	}


	fn set_command(&self, request: WebRequests) -> (Requests, u128) { 

		let msg = request.get_msg().0;
		let id = unsafe{INTERCOM_ID.get_id()};

        match self {
			Commands::TerminalGet => {
				(Requests::TerminalGetText(BasicGetRequest::<Terminal, Text>(ID(id), PhantomData, PhantomData)), id)
            },
			Commands::TerminalSet => {
				(Requests::TerminalSetText(BasicSetRequest::<Terminal, Text>(ID(id), Text(msg.to_string()), PhantomData)), id)
            },
            // Commands::DoorGet => {  
            // },
            // Commands::DoorSet => {  
            // },
            // Commands::AudioGet => {
            // },
			// Commands::AudioSet => {
            // },
            // Commands::CameraGet => {
			// },
            // Commands::CameraSet => {
			// },
            Commands::Unknown => {
				(Requests::TerminalSetText(BasicSetRequest::<Terminal, Text>(ID(id), Text(msg.to_string()), PhantomData)), id)
			}
        }
    }

}



pub fn match_intercom_response(response: Responses, id: u128) -> String{

	let mut message = "".to_string();
	match response {
		Responses::TerminalGetText(_) => {
			if let Responses::TerminalGetText(msg_get) = response {
				assert_eq!(msg_get.get_id().0, id);
				message = msg_get.get_result().unwrap().0;
			}
		}
		Responses::TerminalSetText(_) => {
			if let Responses::TerminalSetText(msg_set) = response {
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
pub fn listen_for_web(request: WsRequest) -> String{

	let new_request = WebRequests(DeviceCommand(request.command), Message(request.message));

	let request_command = new_request.clone();
	let command = Commands::match_command(new_request.get_device_command().0);
	let (setrequest, id): (Requests, u128) = Commands::set_command(&command, request_command);

	// send command to intercom and get reply
	let response = send_command_to_intercom(setrequest);
	let reply_to_web = match_intercom_response(response, id);

	reply_to_web

}

fn send_command_to_intercom(request: Requests) -> Responses {

	let message = thread::spawn(move || {
		
		match TcpStream::connect("127.0.0.1:2000") {

			Ok(mut intercom_stream) => {
				// send command to intercom
				write_to_stream(&mut intercom_stream, &request);
				
				// get reply
				let reply: Responses = read_from_stream(&mut intercom_stream).unwrap();
				reply
			},
			Err(e) => {
				println!("Failed to connect: {}", e);
				process::exit(1);
			}
		}
	});	

	let result = message.join().unwrap();
	result
}












// listens for messages from intercom and relays to web
pub fn listen_for_intercom(sender: Sender<String>) {

	let listener = TcpListener::bind("127.0.0.1:8001").unwrap();
	println!("Server listening on 127.0.0.1:8001");

	for stream in listener.incoming() {	

		if let Err(_) = stream {
            continue;
        }

		let tx = sender.clone();

		thread::spawn(move || {
	
			let mut stream = stream.unwrap();
			// get message from intercom
			let request = read_from_stream(&mut stream).unwrap();
			
			// send message to web 
			tx.send(request).unwrap();
		
		});
	} 
    
    drop(listener);
}





