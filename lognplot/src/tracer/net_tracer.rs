//! Trace metrics over the web.

use super::Tracer;
use crate::net::TcpClient;
use std::sync::Mutex;
use std::time::Instant;

pub struct TcpTracer {
    gui_start_instant: Instant,
    client: Mutex<TcpClient>,
}

impl TcpTracer {
    pub fn new(client: TcpClient) -> Self {
        TcpTracer {
            gui_start_instant: Instant::now(),
            client: Mutex::new(client),
        }
    }
}

impl Tracer for TcpTracer {
    fn log_meta_metric(&self, name: &str, timestamp: Instant, value: f64) {
        let elapsed = timestamp.duration_since(self.gui_start_instant);
        let elapsed_seconds: f64 = elapsed.as_secs_f64();
        if let Err(err) = self
            .client
            .lock()
            .unwrap()
            .send_sample(name, elapsed_seconds, value)
        {
            error!("Error sending metric: {:?}", err);
        }
    }
}
