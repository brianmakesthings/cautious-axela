use super::door::DoorState;
use super::{Device, Shutdown};
use crate::device::door::Door;
use crate::message::{self, ThreadSender};
use crate::message::{Receive, Send, TcpSender, ThreadReceiver};
use crate::request::{BasicSetRequest, Error, Get, GetRequest, Set, SetRequest, ID};
use crate::requests_and_responses::{InternalThreadRequest, Requests, Responses, ThreadRequest};
use openapi::apis::{configuration::Configuration, default_api as twilio_api};
use phonenumber::PhoneNumber;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::Write;
use std::marker::PhantomData;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{env, fs};
use tokio::runtime::{Builder, Runtime};

use sysfs_gpio::Pin;

pub struct KeyPadDevice {
    door_sender: ThreadSender<InternalThreadRequest, Door>,
    sender: TcpSender<Responses>,
    receiver: ThreadReceiver<ThreadRequest>,
    keypad: KeyPad,
    runtime: Runtime,
}

impl KeyPadDevice {
    pub fn new(
        door_sender: ThreadSender<InternalThreadRequest, Door>,
        sender: TcpSender<Responses>,
        receiver: ThreadReceiver<ThreadRequest>,
        keypad: KeyPad,
    ) -> KeyPadDevice {
        let runtime = Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        KeyPadDevice {
            door_sender,
            sender,
            receiver,
            keypad,
            runtime,
        }
    }
}

impl Send<Responses> for KeyPadDevice {
    fn send(&mut self, target: Responses) {
        self.sender.send(target);
    }
}

impl Receive<ThreadRequest> for KeyPadDevice {
    fn receive(&mut self) -> Result<ThreadRequest, message::Error> {
        self.receiver.receive()
    }
}

impl Device<ThreadRequest, Responses> for KeyPadDevice {
    fn handle_command(&mut self, request: ThreadRequest) -> Shutdown {
        let ThreadRequest(request, stream) = request;
        self.sender.set_stream(stream);
        match request {
            Requests::KeyPadGetCode(x) => self
                .sender
                .send(Responses::KeyPadGetCode(x.get_response(&self.keypad))),
            Requests::KeyPadSetCode(x) => self
                .sender
                .send(Responses::KeyPadSetCode(x.get_response(&mut self.keypad))),
            Requests::PhoneGet(x) => self
                .sender
                .send(Responses::PhoneGet(x.get_response(&self.keypad))),
            Requests::PhoneSet(x) => self
                .sender
                .send(Responses::PhoneSet(x.get_response(&mut self.keypad))),
            _ => panic!("Keypad device received invalid request"),
        }
        Shutdown(false)
    }
    fn get_sleep_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(50))
    }
    fn step(&mut self) {
        self.keypad.add_keys();
        let keypad_code = self.keypad.check_candidates();
        match keypad_code {
            CodeType::Code => {
                let internal_request = InternalThreadRequest(Requests::DoorSetState(
                    BasicSetRequest::<Door, DoorState>(ID(0), DoorState::Unlock, PhantomData),
                ));
                self.door_sender.send(internal_request);
            }
            CodeType::Ring if self.keypad.last_rang.elapsed() >= KeyPad::RING_TIMER => {
                self.keypad.last_rang = Instant::now();
                self.runtime.spawn(send_notification(self.keypad.phonenumber.to_string()));
            }
            _ => (),
        }
        if self.keypad.last_pressed.elapsed() >= KeyPad::RESET_TIMER {
            self.keypad.reset_input_keys();
        }
    }
}

#[derive(Clone)]
pub struct KeyPadMatrix {
    rows: [Pin; 4],
    cols: [Pin; 4],
}

impl KeyPadMatrix {
    pub const COL_PINS: [u64; 4] = [66, 67, 69, 68];
    pub const ROW_PINS: [u64; 4] = [3, 2, 15, 115];
    const POS_TO_CHAR: [[char; 4]; 4] = [
        ['1', '2', '3', 'A'],
        ['4', '5', '6', 'B'],
        ['7', '8', '9', 'C'],
        ['*', '0', '#', 'D'],
    ];
    pub fn new(rows: [Pin; 4], cols: [Pin; 4]) -> KeyPadMatrix {
        for pin in rows {
            pin.set_direction(sysfs_gpio::Direction::In).unwrap();
            pin.set_active_low(false).unwrap();
        }
        for pin in cols {
            pin.set_direction(sysfs_gpio::Direction::In).unwrap();
            pin.set_active_low(false).unwrap();
        }
        sleep(Duration::from_millis(500));
        KeyPadMatrix { rows, cols }
    }
    pub fn get_keys_pressed(&self) -> HashSet<char> {
        let mut set = HashSet::new();
        for (i, row_pin) in self.rows.iter().enumerate() {
            row_pin.set_direction(sysfs_gpio::Direction::Out).unwrap();
            row_pin.set_value(0).unwrap();
            for (j, col_pin) in self.cols.iter().enumerate() {
                if col_pin.get_value().unwrap() == 0 {
                    set.insert(KeyPadMatrix::POS_TO_CHAR[i][j]);
                }
            }
            row_pin.set_direction(sysfs_gpio::Direction::In).unwrap();
            row_pin.set_active_low(false).unwrap();
        }
        set
    }
}

#[derive(Clone)]
pub struct CandidateKey {
    data: VecDeque<char>,
    previous_pressed_keys: HashSet<char>,
    is_pressed_keys_initialized: bool,
    capacity: usize,
}

impl CandidateKey {
    const ENDING_KEY: char = '#';
    pub const INITIAL_CAPACITY: usize = 256;
    pub fn new(capacity: usize) -> CandidateKey {
        CandidateKey {
            data: VecDeque::new(),
            previous_pressed_keys: HashSet::new(),
            is_pressed_keys_initialized: false,
            capacity,
        }
    }
    pub fn add_keys(&mut self, keys: HashSet<char>) {
        if !self.is_pressed_keys_initialized {
            self.is_pressed_keys_initialized = true;
            self.previous_pressed_keys = keys;
            return;
        }
        let unpressed_keys = self.previous_pressed_keys.difference(&keys);
        for ch in unpressed_keys {
            self.data.push_back(ch.clone());
            if self.data.len() == self.capacity {
                self.data.pop_front();
            }
        }
        self.previous_pressed_keys = keys;
    }
    pub fn get_candidate_keys(&mut self) -> Vec<String> {
        let candidates = self
            .data
            .iter()
            .collect::<String>()
            .split_inclusive(CandidateKey::ENDING_KEY)
            .filter(|x| x.ends_with(CandidateKey::ENDING_KEY))
            .map(|x| x.trim_end_matches(CandidateKey::ENDING_KEY))
            .map(|x| x.to_owned())
            .collect::<Vec<String>>();
        let length: usize = candidates.iter().map(|x| x.len() + 1).sum();
        for _ in 0..length {
            self.data.pop_front();
        }
        candidates
    }
    pub fn clear(&mut self) {
        self.data.clear();
        self.previous_pressed_keys.clear();
        self.is_pressed_keys_initialized = false;
    }
}

#[derive(Copy, Clone)]
pub enum CodeType {
    Code,
    Ring,
    Invalid,
}

#[derive(Clone)]
pub struct KeyPad {
    code: Code,
    matrix: KeyPadMatrix,
    potential_key: CandidateKey,
    last_pressed: Instant,
    last_rang: Instant,
    phonenumber: PhoneNumber,
}

impl KeyPad {
    pub const RESET_TIMER: Duration = Duration::from_secs(5);
    pub const RING_TIMER: Duration = Duration::from_secs(5);
    const RING: &'static str = "***";
    pub fn new(
        code: Code,
        matrix: KeyPadMatrix,
        potential_key: CandidateKey,
        last_pressed: Instant,
    ) -> KeyPad {
        KeyPad {
            code,
            matrix,
            potential_key,
            last_pressed,
            last_rang: Instant::now() - KeyPad::RING_TIMER,
            phonenumber: phonenumber::parse(
                None,
                env::var("TO_NUMBER").expect("Failed to parse 'to' number"),
            )
            .unwrap(),
        }
    }
    pub fn add_keys(&mut self) {
        let keys = self.matrix.get_keys_pressed();
        if keys.len() != 0 {
            self.last_pressed = Instant::now();
        }
        self.potential_key.add_keys(keys);
    }
    pub fn check_candidates(&mut self) -> CodeType {
        let candidates = self.potential_key.get_candidate_keys();
        let v: Vec<CodeType> = candidates
            .iter()
            .rev()
            .take(1)
            .map(|x| {
                if self.code.is_candidate_valid(x) {
                    CodeType::Code
                } else if x == KeyPad::RING {
                    CodeType::Ring
                } else {
                    CodeType::Invalid
                }
            })
            .collect();
        if v.is_empty() {
            CodeType::Invalid
        } else {
            v[0]
        }
    }
    pub fn get_last_pressed(&self) -> Instant {
        self.last_pressed
    }
    pub fn reset_input_keys(&mut self) {
        self.potential_key.clear();
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Code {
    pub data: String,
}

impl Code {
    const VALID_CHARS: [char; 14] = [
        'A', 'B', 'C', 'D', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
    ];
    const START_UP_CODE_FILE: &'static str = "code";
    fn validate_string(string: &str) -> bool {
        string
            .chars()
            .map(|x| Code::VALID_CHARS.contains(&x))
            .all(|x| x)
    }
    pub fn from_string(string: &str) -> Result<Code, Error> {
        if Code::validate_string(string) {
            return Ok(Code {
                data: string.to_string(),
            });
        }
        Err(Error("Invalid code format allocation".to_string()))
    }
    pub fn is_candidate_valid(&self, candidate: &str) -> bool {
        candidate == self.data
    }
    pub fn new() -> Code {
        let code = fs::read_to_string(Code::START_UP_CODE_FILE)
            .unwrap()
            .trim()
            .to_owned();
        Code { data: code }
    }
}

impl Get<KeyPad, Code> for KeyPad {
    fn get(&self) -> Result<Code, Error> {
        Ok(self.code.clone())
    }
}

impl Set<KeyPad, Code> for KeyPad {
    fn set(&mut self, target: &Code) -> Result<(), Error> {
        let new_code_result = Code::from_string(&target.data);
        match new_code_result {
            Ok(new_code) => {
                self.code = new_code;
                let mut file = File::create(Code::START_UP_CODE_FILE).unwrap();
                file.write_all(self.code.data.as_bytes()).unwrap();
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhoneNumberText(pub String);

impl Get<KeyPad, PhoneNumberText> for KeyPad {
    fn get(&self) -> Result<PhoneNumberText, Error> {
        Ok(PhoneNumberText(self.phonenumber.to_string()))
    }
}

impl Set<KeyPad, PhoneNumberText> for KeyPad {
    fn set(&mut self, target: &PhoneNumberText) -> Result<(), Error> {
        if phonenumber::is_viable(&target.0) {
            self.phonenumber = phonenumber::parse(None, &target.0).unwrap();
            Ok(())
        } else {
            Err(Error("Invalid phonenumber".to_string()))
        }
    }
}

async fn send_notification(to: &str) {
    let account_sid = env::var("TWILIO_ACCOUNT_SID").expect("Failed to parse Account SID");
    let api_key = env::var("TWILIO_API_KEY").expect("Failed to parse API Key");
    let api_key_secret = env::var("TWILIO_API_KEY_SECRET").expect("Failed to parse API Key Secret");
    let from = env::var("TWILIO_PHONE_NUMBER").expect("Failed to parse 'from' number");

    let mut twilio_config = Configuration::default();
    twilio_config.basic_auth = Some((api_key, Some(api_key_secret)));

    let message = twilio_api::create_message(
        &twilio_config,
        &account_sid,
        &to,
        None,
        None,
        None,
        Some("Someone is ringing the bell!"),
        None,
        None,
        Some(&from),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await;

    let _result = match message {
        Ok(result) => result,
        Err(error) => panic!("Something went wrong, {:?}", error),
    };
}
