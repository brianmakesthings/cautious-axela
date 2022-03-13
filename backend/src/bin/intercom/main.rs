use common::build::Build;
use common::device;
use common::device::terminal;
use common::dispatch;
use std::net::TcpListener;
use std::thread;

mod camera;

fn main() {
    // Create
    let (terminal_channel, terminal_device) = terminal::TerminalDevice::build(());
    let dispatcher = dispatch::Dispatcher::build(terminal_channel);
    let terminal_handle = device::launch_device(terminal_device);
    let camera_handle = thread::spawn(camera::mainloop);

    // Start server
    let listener = TcpListener::bind("127.0.0.1:2000").unwrap();
    let dispatch_handle = thread::spawn(|| {
        dispatch::start_server(dispatcher, listener);
    });

    // Clean up
    terminal_handle.join().unwrap();
    dispatch_handle.join().unwrap();
    camera_handle.join().unwrap();
}
