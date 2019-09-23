#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

pub mod gateway {
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct ResourceIdentifier {
        /// The resource ID
        pub rid: String,
    }

    pub fn model_result(model: serde_json::Value) -> serde_json::Value {
        json!({
            "result" :{
                "model" : model
            }
        })
    }

    pub fn error_not_found(msg: &str) -> serde_json::Value {        
        json!({
            "error": {
                "code": "system.notFound",
                "message": msg
            }
        })
    }

    pub fn success_response() -> serde_json::Value {        
        json!({
            "result": null 
        })
    }
}
    

pub mod timer {
    include!(concat!(env!("OUT_DIR"), "/timer.rs"));

    use prost::Message;

    pub const OP_TIMER_TICK: &str = "decs:timer!Tick";

    impl Into<TimerTick> for &[u8] {
        fn into(self) -> TimerTick {
            TimerTick::decode(self).unwrap()
        }
    }

    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct GameLoopTick {
        pub seq_no: u64,
        pub elapsed_ms: u32,
        pub shard: String,
    }

    impl GameLoopTick {
        pub fn from_tick(source: &TimerTick, shard: &str) -> Self {
            GameLoopTick {
                seq_no: source.seq_no as _,
                elapsed_ms: source.elapsed_ms as _,
                shard: shard.to_string(),
            }
        }
    }    
}


pub mod shard {
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct Shard {
        pub name: String,
        pub capacity: u32,
        pub current: u32
    }

    impl Shard {
        pub fn the_void() -> Shard {
            Shard {
                name: "the_void".to_string(),
                capacity: 1_000,
                current: 0
            }
        }
    }
}

pub mod systemmgr {
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct EntityFrame {
        /// Monotonically increasing sequence number
        pub seq_no: u64,
        /// Elapsed time (ms, approx) since the last frame
        pub elapsed_ms: u32,
        /// Id of the shard in which this frame takes place
        pub shard: String,
        /// Entity ID to which the components in the vector belong
        pub entity_id: String,        
    }

    /// Represents a single instance of a component value. For example, player `bob` in the `default`
    /// space might have a `location` value of `{"x": 12, "y": 50}`.
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct ComponentValue {
        /// Component name
        pub name: String,
        /// Space in which this value exists        
        pub value: String,
    }

    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct System {
        /// The name of the system
        pub name: String,
        /// Frame rate, in frames per second, this system prefers getting component updates
        pub framerate: u32,
        /// List of components for which this system has registered for updates (delivered per frame)
        pub components: Vec<String>,        
    }    
}
