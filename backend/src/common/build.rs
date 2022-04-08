use crate::device::terminal;
use crate::device::nfc;
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

impl Build for nfc::NFCDevice {
    type Input = ();
    type Result = (
        message::ThreadSender<ThreadRequest, nfc::NFC>,
        nfc::NFCDevice,
    );
    fn build(_: ()) -> Self::Result {
        let (sender, receiver) = mpsc::channel();
        let thread_receiver = message::ThreadReceiver(receiver, PhantomData);
        let nfc = nfc::NFC();
        let tcp_sender = message::TcpSender(None, PhantomData);
        let nfc_device = nfc::NFCDevice::new(tcp_sender, thread_receiver, nfc);
        let nfc_channel = message::ThreadSender(sender, PhantomData);
        (nfc_channel, nfc_device)
    }
}

impl Build for dispatch::Dispatcher {
    type Result = dispatch::Dispatcher;
    type Input = (
        message::ThreadSender<ThreadRequest, terminal::Terminal>,
        message::ThreadSender<ThreadRequest, nfc::NFC>,
    );
    fn build(input: Self::Input) -> Self::Result {
        dispatch::Dispatcher::new(input.0, input.1)
    }
}
