use common::build::Build;
use common::device;
use common::device::terminal;
use common::device::nfc;
use common::dispatch;
use std::net::TcpListener;
use std::thread;

fn main() {
    // Create
    let (terminal_channel, terminal_device) = terminal::TerminalDevice::build(());
    let (nfc_channel, nfc_device) = nfc::NFCDevice::build(());
    let dispatcher = dispatch::Dispatcher::build(terminal_channel, nfc_channel);
    let terminal_handle = device::launch_device(terminal_device);
    let nfc_handle = device::launch_device(nfc_device);

    // Start server
    let listener = TcpListener::bind("127.0.0.1:2000").unwrap();
    let dispatch_handle = thread::spawn(|| {
        dispatch::start_server(dispatcher, listener);
    });

    // Clean up
    terminal_handle.join().unwrap();
    nfc_handle.join().unwrap();
    dispatch_handle.join().unwrap();
}
