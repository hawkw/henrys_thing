mod collector;

use log::{error, info};

use hyper::{Body, Request, Response, Server};
use tokio;
use futures::{future, Future, Stream};

fn main() {
    pretty_env_logger::init();

    tokio::run(future::lazy(|| {
        let addr = ([127, 0, 0, 1], 3000).into();
        info!("listening; addr={};", addr);

        let (handle, task) = collector::new();
        tokio::spawn(task);
        Server::bind(&addr)
            .serve(move || {
                let handle = handle.clone();
                hyper::service::service_fn(move |req: Request<Body>| {
                    let handle = handle.clone();
                    let f = req.into_body().map(|chunk| {
                        chunk.iter().map(|&c| c).collect::<Vec<u8>>()
                    })
                    .into_future()
                    .map_err(|(e, _trash)| e)
                    .and_then(move |(body, _trash)| {
                        let handle = handle.clone();
                        let n = String::from_utf8(body.unwrap())
                            .map_err(|e| error!("bad body: {:?}", e))
                            .and_then(|b| b.parse::<usize>().map_err(|e| error!("bad thing: {:?}", e)));
                        match n {
                            Ok(n) => future::Either::A(handle.send(n)
                                .map(|result| Response::new(Body::from(format!("{}", result))))
                                .or_else(|_| future::ok(Response::new(Body::from("elizas bullshit broke!"))))),
                            _ => future::Either::B(future::ok(Response::new(Body::from("ur req is bad!")))),
                        }

                    });

                    f
                })
            })
            .map_err(|e| error!("server machine broke: {}", e))
    }));

}
