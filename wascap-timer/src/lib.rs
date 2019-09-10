#[macro_use]
extern crate wascap_codec;

#[macro_use]
extern crate log;

use crossbeam_channel::tick;
use decscloud_codec as decs;
use prost::Message;
use std::cmp;
use std::error::Error;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};
use wascap_codec as codec;
use wascap_codec::capabilities::{CapabilityProvider, Dispatcher, NullDispatcher};

capability_provider!(TimerProvider, TimerProvider::new);

const CAPABILITY_ID: &str = "decs:timer";
const ENV_INTERVAL: &str = "TIMER_INTERVAL_FPS";
const ENV_MAX_FPS: &str = "TIMER_MAX_FPS";
const MAX_FPS_DEFAULT: u32 = 10;
const INTERVAL_DEFAULT: u32 = 1;

pub struct TimerProvider {
    dispatcher: Arc<RwLock<Box<dyn Dispatcher>>>,
    interval: u32,
    max_fps: u32,
}

impl TimerProvider {
    pub fn new() -> TimerProvider {
        Self::default()
    }

    fn get_delay_ms(&self) -> u64 {
        let delay_at_max = 1000_u64 / u64::from(self.max_fps);
        let desired_delay = 1000_u64 / u64::from(self.interval);

        cmp::max(desired_delay, delay_at_max)
    }
}

impl Default for TimerProvider {
    fn default() -> Self {
        let interval = match std::env::var(ENV_INTERVAL) {
            Ok(v) => v.parse::<u32>().unwrap(),
            Err(_) => INTERVAL_DEFAULT,
        };

        let max_fps = match std::env::var(ENV_MAX_FPS) {
            Ok(v) => v.parse::<u32>().unwrap(),
            Err(_) => MAX_FPS_DEFAULT,
        };

        TimerProvider {
            dispatcher: Arc::new(RwLock::new(Box::new(NullDispatcher::new()))),
            interval,
            max_fps,
        }
    }
}

impl CapabilityProvider for TimerProvider {
    fn capability_id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn configure_dispatch(
        &self,
        dispatcher: Box<dyn Dispatcher>,
        _id: codec::capabilities::ModuleIdentity,
    ) -> Result<(), Box<dyn Error>> {
        env_logger::init();
        info!("Dispatcher received.");

        let mut lock = self.dispatcher.write().unwrap();
        *lock = dispatcher;

        // 1000 / fps
        // 1 FPS = 1000/1 = 1000
        // 10 FPS = 1000/10 = 100
        // and so on

        let delay = self.get_delay_ms();
        info!("Starting timer with {}ms delay", delay);

        let mut start = Instant::now();
        let ticker = tick(Duration::from_millis(delay));
        let d = self.dispatcher.clone();
        let mut seq = 0;

        std::thread::spawn(move || loop {
            ticker.recv().unwrap();
            let d = d.read().unwrap();
            let tick = decs::timer::TimerTick {
                seq_no: seq,
                elapsed_ms: start.elapsed().as_millis() as i32,
            };
            let mut buf = Vec::new();
            tick.encode(&mut buf).unwrap();
            seq += 1;
            start = Instant::now();
            d.dispatch(decs::timer::OP_TIMER_TICK, &buf).unwrap();
        });

        Ok(())
    }

    fn name(&self) -> &'static str {
        "dECS Cloud Timer Provider"
    }

    fn handle_call(&self, op: &str, _msg: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        info!("Received host call, operation - {}, Ignored.", op);

        Ok(vec![])
    }
}

#[cfg(test)]
mod test {
    use super::{TimerProvider, ENV_INTERVAL};

    #[test]
    fn test_delay_default() {
        std::env::remove_var(ENV_INTERVAL);
        let t = TimerProvider::new();
        assert_eq!(t.get_delay_ms(), 1000);
    }

    #[test]
    fn test_explicit_delay() {
        std::env::set_var(ENV_INTERVAL, "4"); // ask for 4 FPS
        let t = TimerProvider::new();
        assert_eq!(t.get_delay_ms(), 250);
    }

    #[test]
    fn test_limited_delay() {
        std::env::set_var(ENV_INTERVAL, "15"); // ask for 15, which exceeds cap
        let t = TimerProvider::new();
        assert_eq!(t.get_delay_ms(), 100);
    }
}
