use i2cdev::core::I2CDevice;
use i2cdev::linux::LinuxI2CDevice;
use std::io::Result as IOResult;
use std::marker::PhantomData;
use std::process::{Command, Output};
use std::thread::sleep;
use std::time::Duration;

use super::{Device, Shutdown};
use crate::device::door::{Door, DoorState};
use crate::message::{self, ThreadSender};
use crate::message::{Receive, Send, TcpSender, ThreadReceiver};
use crate::request::{BasicSetRequest, Error, Get, GetRequest, Set, SetRequest, ID};
use crate::requests_and_responses::{InternalThreadRequest, Requests, Responses, ThreadRequest};
use serde::{Deserialize, Serialize};

const PN532_ADDRESS: u8 = 0x48 >> 1;

// commands
#[allow(unused)]
enum Commands {
    Diagnose = 0x00,
    GetFirmwareVersion = 0x02,
    GetGeneralStatus = 0x04,
    ReadRegister = 0x06,
    WriteRegister = 0x08,
    ReadGPIO = 0x0C,
    WriteGPIO = 0x0E,
    SetSerialBaudRate = 0x10,
    SetParameters = 0x12,
    SAMConfiguration = 0x14,
    PowerDown = 0x16,
    RFConfiguration = 0x32,
    RFRegulationTest = 0x58,
    InJumpForDEP = 0x56,
    InJumpForPSL = 0x46,
    InListPassiveTarget = 0x4A,
    InATR = 0x50,
    InPSL = 0x4E,
    InDataExchange = 0x40,
    InCommunicateThru = 0x42,
    InDeselect = 0x44,
    InRelease = 0x52,
    InSelect = 0x54,
    InAutoPoll = 0x60,
    TgInitAsTarget = 0x8C,
    TgSetGeneralBytes = 0x92,
    TgGetData = 0x86,
    TgSetData = 0x8E,
    TgSetMetaData = 0x94,
    TgGetInitiatorCommand = 0x88,
    TgResponseToInitiator = 0x90,
    TgGetTargetStatus = 0x8A,
}

#[allow(unused)]
enum CardTypes {
    IsoTypeA = 0x00,
    FeliCa212 = 0x01,
    FeliCa424 = 0x02,
    IsoTypeB = 0x03,
    Jewel = 0x04,
}

#[derive(Clone)]
pub struct NFCdev {
    ids: Vec<Vec<u8>>,
}

pub struct NFCDevice {
    door_sender: ThreadSender<InternalThreadRequest, Door>,
    sender: TcpSender<Responses>,
    receiver: ThreadReceiver<ThreadRequest>,
    nfc: NFCdev,
}

impl Send<Responses> for NFCDevice {
    fn send(&mut self, target: Responses) {
        self.sender.send(target);
    }
}

impl Receive<ThreadRequest> for NFCDevice {
    fn receive(&mut self) -> Result<ThreadRequest, message::Error> {
        self.receiver.receive()
    }
}

impl Device<ThreadRequest, Responses> for NFCDevice {
    fn handle_command(&mut self, request: ThreadRequest) -> Shutdown {
        let ThreadRequest(request, stream) = request;
        self.sender.set_stream(stream);
        match request {
            Requests::NFCGetID(x) => self
                .sender
                .send(Responses::NFCGetID(x.get_response(&self.nfc))),
            Requests::NFCSetID(x) => self
                .sender
                .send(Responses::NFCSetID(x.get_response(&mut self.nfc))),
            _ => panic!("NFC device received invalid request"),
        }
        Shutdown(false)
    }
    fn get_sleep_duration(&self) -> Option<Duration> {
        Some(Duration::from_millis(200))
    }
    fn step(&mut self) {
        let uid = self.nfc.get_uid();
        let uid = match uid {
            Ok(id) => id,
            Err(_) => {
                return;
            }
        };

        let ids = self.nfc.ids.clone();
        for data in ids {
            if data == uid[0] {
                println!("Card Authenticattion Succeeded. Opening lock.");
                // Unlock door
                let internal_request = InternalThreadRequest(Requests::DoorSetState(
                    BasicSetRequest::<Door, DoorState>(ID(0), DoorState::Unlock, PhantomData),
                ));
                self.door_sender.send(internal_request);
                sleep(Duration::from_millis(1000));
            }
        }
    }
}

fn enable_bus() -> IOResult<Output> {
    Command::new("config-pin")
        .arg("P9_19")
        .arg("i2c")
        .output()?;
    Command::new("config-pin").arg("P9_20").arg("i2c").output()
}

impl NFCDevice {
    pub fn new(
        door_sender: ThreadSender<InternalThreadRequest, Door>,
        sender: TcpSender<Responses>,
        receiver: ThreadReceiver<ThreadRequest>,
        nfc: NFCdev,
    ) -> NFCDevice {
        return NFCDevice {
            door_sender,
            sender,
            receiver,
            nfc,
        };
    }
}

impl NFCdev {
    pub fn new() -> Self {
        match enable_bus() {
            _ => (),
        }
        let ids_vec = Vec::new();
        let mut nfc = Self { ids: ids_vec };
        match nfc.init_nfc() {
            _ => (),
        }
        nfc
    }

    fn push(&mut self, new_id: Vec<u8>) {
        self.ids.push(new_id);
    }

    pub fn init_nfc(&mut self) -> IOResult<()> {
        self.send_command_to_nfcdev(&[Commands::SAMConfiguration as u8, 0x01])?;
        self.sync_packets()
    }

    pub fn get_uid(&mut self) -> IOResult<Vec<Vec<u8>>> {
        self.send_command_to_nfcdev(&[
            Commands::InListPassiveTarget as u8,
            0x01,
            CardTypes::IsoTypeA as u8,
        ])?;

        self.sync_packets()?;
        let reply = self.receive_from_nfcdev()?;
        let reply_length = reply.len();

        if reply_length < 5 {
            return Ok(Vec::new());
        }

        let mut index = 6;
        let mut id = Vec::new();

        if index >= reply_length {
            return Ok(Vec::new());
        }

        let id_length = reply[index] as usize;
        index += 1;
        if index + id_length > reply_length {
            return Ok(Vec::new());
        }
        id.push(reply[index..index + id_length].to_vec());
        Ok(id)
    }

    fn sync_packets(&mut self) -> IOResult<()> {
        sleep(Duration::from_millis(1));
        let mut i2cdev = LinuxI2CDevice::new("/dev/i2c-2", PN532_ADDRESS.into()).unwrap();

        for _ in 0..5 {
            let mut data = [0u8; 128];
            i2cdev.read(&mut data)?;
            let mut index = 0;

            for j in 0..data.len() {
                match index {
                    0 | 1 => {
                        if data[j] == 0x00 {
                            index += 1;
                            continue;
                        } else {
                            index = 0;
                        }
                    }
                    2 => {
                        if data[j] == 0xFF {
                            index += 1;
                            continue;
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Ack Response Err1",
                            )
                            .into());
                        }
                    }
                    3 => {
                        if data[j] == 0x00 {
                            return Ok(());
                        } else if data[j] as u8 == 0xFF {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Nack Error",
                            )
                            .into());
                        } else if data[j] as u8 == 0x01 {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "App Error",
                            )
                            .into());
                        }
                    }
                    _ => {
                        index = 0;
                    }
                }
            }
        }

        Err(std::io::Error::new(std::io::ErrorKind::Other, "timeout").into())
    }

    fn send_command_to_nfcdev(&mut self, data: &[u8]) -> IOResult<()> {
        let length = data.len() as u8;
        let checksum_lcs = !length as u8;
        let mut i2cdev = LinuxI2CDevice::new("/dev/i2c-2", PN532_ADDRESS.into()).unwrap();

        let tfi = 0xd4;
        let mut checksum_dcs = tfi;
        for i in data {
            checksum_dcs = checksum_dcs + i;
        }

        checksum_dcs = !(checksum_dcs & 0xFF) as u8 + 1;
        let mut frame = vec![0x00, 0x00, 0xff, length + 1, checksum_lcs, tfi];
        frame.extend_from_slice(data);
        frame.push(checksum_dcs);
        frame.push(0x00);

        match i2cdev.write(&frame) {
            _ => Ok(()),
        }
    }

    fn receive_from_nfcdev(&mut self) -> IOResult<Vec<u8>> {
        let mut i2cdev = LinuxI2CDevice::new("/dev/i2c-2", PN532_ADDRESS.into()).unwrap();

        for _ in 0..10 {
            sleep(Duration::from_millis(1));
            let mut data = [0u8; 128];
            i2cdev.read(&mut data)?;
            let mut index = 0;

            for j in 0..data.len() {
                match index {
                    0 | 1 => {
                        if data[j] == 0x00 {
                            index += 1;
                            continue;
                        } else {
                            index = 0;
                        }
                    }
                    2 => {
                        if data[j] == 0xFF {
                            index += 1;
                            continue;
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Ack Response Err2",
                            )
                            .into());
                        }
                    }
                    3 => match data[j] {
                        0x01 => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "App Error",
                            )
                            .into());
                        }
                        size => {
                            return Ok(data[j + 3..j + 3 + (size as usize - 1)].to_vec());
                        }
                    },
                    _ => {
                        index = 0;
                    }
                }
            }
        }
        Err(std::io::Error::new(std::io::ErrorKind::Other, "timeout").into())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NFCids(pub String);

impl Set<NFCdev, NFCids> for NFCdev {
    fn set(&mut self, _target: &NFCids) -> Result<(), Error> {
        println!("Scanning for new card...");
        loop {
            let uid = self.get_uid();
            let uid = match uid {
                Ok(id) => id,
                Err(_) => {
                    continue;
                }
            };
            self.push(uid[0].clone());
            println!("Added new card id");
            sleep(Duration::from_millis(1000));
            break;
        }
        Ok(())
    }
}

// Blocking
impl Get<NFCdev, NFCids> for NFCdev {
    fn get(&self) -> Result<NFCids, Error> {
        let str = format!("ids = {:x?}", self.ids);
        Ok(NFCids(str))
    }
}
