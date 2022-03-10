use crate::device::terminal;
use crate::dispatch;
use crate::message;
use crate::requests_and_responses::ThreadRequest;
use std::marker::PhantomData;
use std::sync::mpsc;

pub trait Build {
    type Result;
    type Input;
    fn build(input: Self::Input) -> Self::Result;
}

impl Build for terminal::TerminalDevice {
    type Input = ();
    type Result = (
        message::ThreadSender<ThreadRequest, terminal::Terminal>,
        terminal::TerminalDevice,
    );
    fn build(_: ()) -> Self::Result {
        let (sender, receiver) = mpsc::channel();
        let thread_receiver = message::ThreadReceiver(receiver, PhantomData);
        let terminal = terminal::Terminal();
        let tcp_sender = message::TcpSender(None, PhantomData);
        let terminal_device = terminal::TerminalDevice::new(tcp_sender, thread_receiver, terminal);
        let terminal_channel = message::ThreadSender(sender, PhantomData);
        (terminal_channel, terminal_device)
    }
}

impl Build for dispatch::Dispatcher {
    type Result = dispatch::Dispatcher;
    type Input = message::ThreadSender<ThreadRequest, terminal::Terminal>;
    fn build(input: Self::Input) -> Self::Result {
        dispatch::Dispatcher::new(input)
    }
}
