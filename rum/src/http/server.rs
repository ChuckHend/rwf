use super::{Error, Handler, Path, Request, Response, ToResource};
use crate::controller::Controller;

use colored::Colorize;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tracing::{debug, info};

pub struct Server {
    handlers: Arc<Vec<Handler>>,
}

impl Server {
    pub fn new(handlers: Vec<Handler>) -> Self {
        Server {
            handlers: Arc::new(handlers),
        }
    }

    pub async fn launch(self) -> Result<(), Error> {
        let listener = TcpListener::bind("0.0.0.0:8000").await?;

        loop {
            let (mut stream, peer_addr) = listener.accept().await?;
            let handlers = self.handlers.clone();
            let mut found = false;

            tokio::spawn(async move {
                debug!("HTTP new connection from {:?}", peer_addr);

                loop {
                    let request = match Request::read(&mut stream).await {
                        Ok(request) => request,
                        Err(err) => {
                            debug!("client {:?} disconnected: {:?}", peer_addr, err);
                            return;
                        }
                    };

                    for handler in handlers.iter() {
                        if request.path().matches(handler.path()) {
                            found = true;
                            let start = Instant::now();
                            let response = handler.handle(&request).await.unwrap();
                            let duration = Instant::now() - start;
                            Self::log(
                                request.path(),
                                handler.controller_name(),
                                &response,
                                duration,
                            );
                            response.send(&mut stream).await.unwrap();
                        }
                    }

                    if !found {
                        Response::not_found().send(&mut stream).await.unwrap();
                    }
                }
            });
        }
    }

    fn log(path: &Path, controller_name: &str, response: &Response, duration: Duration) {
        info!(
            "{} {} {} ({:.3} ms)",
            controller_name
                .split("::")
                .skip(1)
                .collect::<Vec<_>>()
                .join("::")
                .green(),
            path.path().purple(),
            response.status().code(),
            duration.as_secs_f64() * 1000.0,
        );
    }
}
