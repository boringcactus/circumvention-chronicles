use std::sync::{Mutex, Arc};
use std::thread;
use super::support;

use futures::future::Future;
use futures::sync::oneshot;
use hyper;
use hyper::StatusCode;
use hyper::server::{Http, Request, Response, Service};

pub struct GameServer {
    state: Arc<Mutex<support::GameState>>,
}

impl GameServer {
    pub fn new(state: Arc<Mutex<support::GameState>>) -> Self {
        return GameServer {
            state
        }
    }

    pub fn run(state: Arc<Mutex<support::GameState>>, kill_receiver: oneshot::Receiver<()>) {
        let addr = "127.0.0.1:9866".parse().unwrap();
        let server = Http::new().bind(&addr, move || Ok(Self::new(Arc::clone(&state)))).unwrap();
        server.run_until(kill_receiver.map_err(|_| ())).unwrap();
    }
}

impl Service for GameServer {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = Box<Future<Item=Self::Response, Error=Self::Error>>;

    fn call(&self, req: Request) -> Self::Future {
        let state_mutex = Arc::clone(&self.state);
        let (p, c) = oneshot::channel::<Self::Response>();
        thread::spawn(move || {
            let mut state = state_mutex.lock().unwrap();
            if req.path() == "/favicon.ico" {
                p.send(Response::new().with_status(StatusCode::NotFound))
            } else {
                p.send(state.handle_request(req.method(), req.path()))
            }.unwrap();
        });
        // The errors really shouldn't happen, so it's OK that they aren't handled reasonably. Right?
        Box::new(c.map_err(|_| hyper::Error::Status))
    }
}
