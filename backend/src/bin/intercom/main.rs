use common::build::Build;
use common::device;
use common::device::terminal;
use common::device::door;
use common::dispatch;
use std::net::TcpListener;
use std::thread;

fn main() {
    // Create
    let (terminal_channel, terminal_device) = terminal::TerminalDevice::build(());
    let (door_channel, door_device) = door::DoorDevice::build(());
    let dispatcher = dispatch::Dispatcher::build((terminal_channel, door_channel));
    let terminal_handle = device::launch_device(terminal_device);
    let door_handle = device::launch_device(door_device);

    // Start server
    let listener = TcpListener::bind("127.0.0.1:2000").unwrap();
    println!("Listening on 192.168.7.2:2000");
    let dispatch_handle = thread::spawn(|| {
        dispatch::start_server(dispatcher, listener);
    });

    // Clean up
    terminal_handle.join().unwrap();
    door_handle.join().unwrap();
    dispatch_handle.join().unwrap();
}
