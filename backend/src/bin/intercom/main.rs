use common::{device, dispatch, message};
use std::thread;
use std::{marker::PhantomData, net::TcpListener, sync::mpsc};

fn main() {
    let (sender, receiver) = mpsc::channel();
    let terminal = device::Terminal();
    let thread_receiver = message::ThreadReceiver(receiver, PhantomData);
    let terminal_handle = thread::spawn(|| {
        device::driver(thread_receiver, terminal);
    });
    let terminal_channel = message::ThreadSender(sender, PhantomData);
    let dispatcher = dispatch::Dispatcher::new(terminal_channel);
    let listener = TcpListener::bind("127.0.0.1:2000").unwrap();
    let dispatch_handle = thread::spawn(|| {
        dispatch::start_server(dispatcher, listener);
    });
    terminal_handle.join().unwrap();
    dispatch_handle.join().unwrap();
}
