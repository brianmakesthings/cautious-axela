use common::build::Build;
use common::device;
use common::device::door;
use common::device::keypad;
use common::device::terminal;
use common::device::nfc;
use common::dispatch;
use std::net::TcpListener;
use std::thread;

fn main() {
    // Create
    let (terminal_channel, terminal_device) = terminal::TerminalDevice::build(());
    let (door_channel, door_internal_channel, door_device) = door::DoorDevice::build(());
    let (nfc_channel, nfc_device) = nfc::NFCDevice::build(door_internal_channel.clone());
    let (keypad_channel, keypad_device) = keypad::KeyPadDevice::build(door_internal_channel);
    let dispatcher = dispatch::Dispatcher::build((terminal_channel, door_channel, keypad_channel, nfc_channel));
    let terminal_handle = device::launch_device(terminal_device);
    let nfc_handle = device::launch_device(nfc_device);
    let door_handle = device::launch_device(door_device);
    let keypad_handle = device::launch_device(keypad_device);

    // Start server
    let listener = TcpListener::bind("0.0.0.0:2000").unwrap();
    println!("Listening on 192.168.7.2:2000");
    let dispatch_handle = thread::spawn(|| {
        dispatch::start_server(dispatcher, listener);
    });

    // Clean up
    terminal_handle.join().unwrap();
    door_handle.join().unwrap();
    nfc_handle.join().unwrap();
    keypad_handle.join().unwrap();
    dispatch_handle.join().unwrap();
}
