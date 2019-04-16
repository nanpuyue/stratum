use std::net::TcpStream as StdTcpStream;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use bytes::Bytes;
use futures::stream::Stream;
use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::{Async::*, Future, Poll};
use tokio::codec::{Decoder, LinesCodec};
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::reactor::Handle;

use super::util::{Notify, SinkHook};
use super::work::*;

pub use self::message::*;

mod checker;
mod message;
mod reader;

#[derive(Debug)]
pub struct WorkStream(pub Receiver<Work>);

impl Stream for WorkStream {
    type Item = Work;
    type Error = ();

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let mut item: Async<Option<Self::Item>> = NotReady;
        loop {
            match self.0.poll() {
                Ok(Ready(w)) => {
                    if let Ready(Some(work)) = item {
                        info!("=> drop work (id: {})!", work.id);
                    }

                    item = Async::Ready(w);
                }
                Ok(NotReady) => return Ok(item),
                Err(_) => return Err(()),
            }
        }
    }
}

pub struct Pool {
    addr: String,
    reader: Option<Receiver<String>>,
    writer: Option<Sender<String>>,
    pub xnonce: Arc<Mutex<(Bytes, usize)>>,
    pub work_channel: (Sender<Work>, Receiver<Work>),
    pub work_notify: Notify,
    pub vermask: Arc<Mutex<Option<u32>>>,
    pub diff: Arc<Mutex<f64>>,
    pub last_active: Arc<Mutex<Instant>>,
}

impl Pool {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: String::from(addr),
            reader: None,
            writer: None,
            xnonce: Arc::new(Mutex::new((Bytes::new(), 0))),
            work_channel: channel(4),
            work_notify: Notify::default(),
            vermask: Arc::new(Mutex::new(None)),
            diff: Arc::new(Mutex::new(1.0)),
            last_active: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn connect(&mut self) -> impl Future<Item = (), Error = ()> + Send {
        let (reader_tx, reader_rx) = channel::<String>(16);
        self.reader = Some(reader_rx);

        let (writer_tx, writer_rx) = channel::<String>(16);
        self.writer = Some(writer_tx);

        let tcpstream = TcpStream::from_std(
            StdTcpStream::connect(&self.addr).expect("tcp connect err!"),
            &Handle::default(),
        )
        .unwrap();

        tcpstream.set_nodelay(true).expect("set_nodelay err!");

        let (sink, stream) = LinesCodec::new().framed(tcpstream).split();

        let last_active = self.last_active.clone();
        let reader = stream
            .inspect(move |line| {
                debug!("recv: {}", line);
                *last_active.lock().unwrap() = Instant::now();
            })
            .map_err(|e| error!("recv from pool err: {:?}", e))
            .forward(reader_tx.sink_map_err(|e| error!("send data to channel err: {:?}", e)));
        let reader = reader.select2(self.reader());

        let writer = writer_rx.inspect(|line| debug!("send: {}", line)).forward(
            SinkHook::new(sink, || debug!("data sent!"))
                .sink_map_err(|e| error!("send to pool err: {:?}", e)),
        );

        reader.select2(writer).map(drop).map_err(drop)
    }

    pub fn sender(&mut self) -> Sender<String> {
        self.writer.clone().unwrap()
    }

    pub fn receiver(&mut self) -> Receiver<String> {
        self.reader.take().unwrap()
    }

    pub fn send<T: serde::Serialize>(&mut self, msg: T) -> impl Future {
        self.sender()
            .send(serde_json::to_string(&msg).unwrap())
            .and_then(|_| Ok(()))
    }

    pub fn subscribe(&mut self) {
        let msg = Action {
            id: Some(1),
            method: String::from("mining.subscribe"),
            params: Params::None([]),
        };
        let _ = self.send(&msg).wait();
    }

    pub fn authorize(&mut self, user: &str, pass: &str) {
        let msg = Action {
            id: Some(2),
            method: String::from("mining.authorize"),
            params: Params::User([String::from(user), String::from(pass)]),
        };
        let _ = self.send(&msg).wait();
    }

    pub fn configure(&mut self, exts: Vec<String>, ext_params: serde_json::Value) {
        let msg = Action {
            id: Some(1),
            method: String::from("mining.configure"),
            params: Params::Config(exts, ext_params),
        };

        let _ = self.send(&msg).wait();
    }
}
