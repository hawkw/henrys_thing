use futures::{stream::Stream, sync::{mpsc, oneshot}, Future, Poll, Async, try_ready};
use log::{error, info, trace, warn};

#[derive(Debug)]
pub struct Collector {
    reqs: Vec<Request>,
    req_rx: mpsc::UnboundedReceiver<Request>,
}

#[derive(Debug, Clone)]
pub struct Handle {
    tx: mpsc::UnboundedSender<Request>,
}

#[derive(Debug)]
struct Request {
    value: usize,
    tx: oneshot::Sender<usize>,
}

pub fn new() -> (Handle, Collector) {
    // TODO: hen ry if u want to, u should put a bound on the mpsc.
    let (tx, req_rx) = mpsc::unbounded();
    let collector = Collector {
        reqs: Vec::new(),
        req_rx,
    };
    let handle = Handle { tx };
    (handle, collector)
}

impl Handle {
    pub fn send(&self, value: usize) -> impl Future<Item = usize, Error = ()> {
        let (tx, rx) = oneshot::channel();
        let req = Request {
            value, tx
        };
        self.tx.unbounded_send(req)
            .expect("unbounded send shouldnt fail, smth is bad");
        rx.map_err(|e| {
            error!("collector machine broke: {:?}", e);
        })
    }
}

impl Future for Collector {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let poll = self.req_rx.poll().map_err(|e| {
                error!("sender done goofed: {:?}", e);
            });
            match try_ready!(poll) {
                None => {
                    info!("collector finished");
                    return Ok(Async::Ready(()));
                }
                Some(req) => {
                    trace!("recieved {:?}", req);
                    self.reqs.push(req);
                }
            };
            if self.reqs.len() == 8 {
                let sum: usize = self.reqs.iter().map(|req| req.value).sum();
                for req in self.reqs.drain(..) {
                    let _ = req.tx.send(sum)
                        .map_err(|_| warn!("tx dropped"));
                }
            }
        }
    }
}
