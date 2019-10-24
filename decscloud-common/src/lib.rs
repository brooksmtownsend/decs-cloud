#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

pub mod gateway {
    //! Support for the RES protocol (e.g. the RESgate server)

    /// The RES protocol uniquely identifies all resources with a Resource ID
    #[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
    pub struct ResourceIdentifier {
        /// The resource ID
        pub rid: String,
    }

    /// Generates a positive RES protocol result containing a model
    pub fn model_result(model: serde_json::Value) -> serde_json::Value {
        json!({
            "result" :{
                "model" : model
            }
        })
    }

    /// Generates a RES protocol error indicating not found (e.g. HTTP 404)
    pub fn error_not_found(msg: &str) -> serde_json::Value {
        json!({
            "error": {
                "code": "system.notFound",
                "message": msg
            }
        })
    }

    /// Generates a RES protocol error indicating invalid parameters (e.g. HTTP bad request)
    pub fn error_invalid_params(msg: &str) -> serde_json::Value {
        json!({
            "error": {
                "code": "system.invalidParams",
                "message": msg
            }
        })
    }

    /// Generates a RES protocol success response with no payload
    pub fn success_response() -> serde_json::Value {
        json!({ "result": null })
    }

    /// Represents the intent of a RES protocol request as described by a message broker subject
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    pub enum ResProtocolRequest {
        Get(String),
        New(String),
        Set(String),
        Delete(String),
        Access(String),
        Call(String, String),
        Unknown,
    }

    impl ToString for ResProtocolRequest {
        fn to_string(&self) -> String {
            match self {
                ResProtocolRequest::Get(resid) => format!("get.{}", resid),
                ResProtocolRequest::New(resid) => format!("call.{}.new", resid),
                ResProtocolRequest::Set(resid) => format!("call.{}.set", resid),
                ResProtocolRequest::Delete(resid) => format!("call.{}.delete", resid),
                ResProtocolRequest::Access(resid) => format!("access.{}", resid),
                ResProtocolRequest::Call(resid, method) => format!("call.{}.{}", resid, method),
                ResProtocolRequest::Unknown => "??".to_string(),
            }
        }
    }

    impl From<&str> for ResProtocolRequest {
        fn from(source: &str) -> Self {
            if source.starts_with("get.") {
                ResProtocolRequest::Get(source[4..].to_string())
            } else if source.starts_with("call.") && source.ends_with(".new") {
                ResProtocolRequest::New(source[5..=source.len() - 5].to_string())
            } else if source.starts_with("call.") && source.ends_with(".set") {
                ResProtocolRequest::Set(source[5..=source.len() - 5].to_string())
            } else if source.starts_with("access.") {
                ResProtocolRequest::Access(source[7..].to_string())
            } else if source.ends_with("delete") {
                ResProtocolRequest::Delete(source[5..=source.len() - 8].to_string())
            } else if source.starts_with("call.") {
                // a call that isn't new or set
                let tokens: Vec<&str> = source.split('.').collect();
                let rid = tokens[1..tokens.len() - 1].join(".");
                let method = tokens[tokens.len() - 1];
                ResProtocolRequest::Call(rid, method.to_string())
            } else {
                ResProtocolRequest::Unknown
            }
        }
    }
}

pub mod timer {
    //! Support for timer-driven game loops and ticks
    include!(concat!(env!("OUT_DIR"), "/timer.rs"));

    use prost::Message;

    /// The Waxosuit operation name for a timer tick
    pub const OP_TIMER_TICK: &str = "decs:timer!Tick";

    impl Into<TimerTick> for &[u8] {
        fn into(self) -> TimerTick {
            TimerTick::decode(self).unwrap()
        }
    }

    /// Represents a single tick of the game loop. This will be emitted by a game loop component
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct GameLoopTick {
        /// Monotonically increasing sequence number of the tick
        pub seq_no: u64,
        /// Elapsed time (in ms) since the last tick was produced. This should be identical to frame-rate unless a system is lagging
        pub elapsed_ms: u32,
        /// The name/ID of the shard for which this tick is bound
        pub shard: String,
    }

    impl GameLoopTick {
        /// Converts a Waxosuit timer tick into a game loop tick
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
    //! Support for Shard data serialization

    /// Represents a shard, or a logical segmentation of the game
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct Shard {
        /// The unique name of the shard
        pub name: String,
        /// The capacity (number of _components_) of the shard
        pub capacity: u32,
        /// Current number of component values contained within the shard
        #[serde(default)]
        pub current: u32,
    }

    impl Shard {
        /// Produce an empty shard called the void
        pub fn the_void() -> Shard {
            Shard {
                name: "the_void".to_string(),
                capacity: 1_000,
                current: 0,
            }
        }
    }
}

pub mod systemmgr {
    //! Support for types related to system management

    /// Represents a single frame of work dispatched to a system by a system manager when the target system is ready to receive
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct EntityFrame {
        /// Monotonically increasing sequence number
        pub seq_no: u64,
        /// Elapsed time (ms, approx) since the last frame
        pub elapsed_ms: u32,
        /// ID of the shard in which this frame takes place
        pub shard: String,
        /// Entity ID to which this frame applies
        pub entity_id: String,
    }

    /// Represents a dECS Cloud System (e.g. _physics_ or _combat_ or _navigation_)
    #[derive(Debug, Serialize, Deserialize, Default)]
    pub struct System {
        /// The name of the system
        pub name: String,
        /// Rate, in frames per second, this system prefers receiving game loop dispatch frames
        pub framerate: u32,
        /// List of components for which this system has registered for updates. This list is an AND - a system will not receive a frame update unless a given entity in a given shard has ALL of the listed components
        pub components: Vec<String>,
    }
}

#[cfg(test)]
mod test {
    use super::gateway::ResProtocolRequest;

    #[test]
    fn test_resprotocol_roundtrip() {
        let mut subject = "call.decs.components.the_void.player1.radar_contacts.new";
        let mut req = ResProtocolRequest::from(subject);
        assert_eq!(
            req,
            ResProtocolRequest::New("decs.components.the_void.player1.radar_contacts".into())
        );
        assert_eq!(
            req.to_string(),
            "call.decs.components.the_void.player1.radar_contacts.new"
        );

        subject = "call.decs.components.the_void.player1.radar_contacts.delete";
        req = ResProtocolRequest::from(subject);
        assert_eq!(
            req,
            ResProtocolRequest::Delete("decs.components.the_void.player1.radar_contacts".into())
        );
        assert_eq!(
            req.to_string(),
            "call.decs.components.the_void.player1.radar_contacts.delete"
        );

        subject = "get.decs.components.the_void.player1.radar_contacts.1";
        req = ResProtocolRequest::from(subject);
        assert_eq!(
            req,
            ResProtocolRequest::Get("decs.components.the_void.player1.radar_contacts.1".into())
        );
        assert_eq!(
            req.to_string(),
            "get.decs.components.the_void.player1.radar_contacts.1"
        );

        subject = "call.decs.components.the_void.player1.position.set";
        req = ResProtocolRequest::from(subject);
        assert_eq!(
            req,
            ResProtocolRequest::Set("decs.components.the_void.player1.position".into())
        );
        assert_eq!(
            req.to_string(),
            "call.decs.components.the_void.player1.position.set"
        );

        subject = "access.decs.components.the_void.player1.radar_contacts.1";
        req = ResProtocolRequest::from(subject);
        assert_eq!(
            req,
            ResProtocolRequest::Access("decs.components.the_void.player1.radar_contacts.1".into())
        );
        assert_eq!(
            req.to_string(),
            "access.decs.components.the_void.player1.radar_contacts.1"
        );

        subject = "call.decs.shard.the_void.set";
        req = ResProtocolRequest::from(subject);
        assert_eq!(req, ResProtocolRequest::Set("decs.shard.the_void".into()));
        assert_eq!(req.to_string(), "call.decs.shard.the_void.set");

        subject = "call.decs.shard.the_void.incr";
        req = ResProtocolRequest::from(subject);
        assert_eq!(
            req,
            ResProtocolRequest::Call("decs.shard.the_void".into(), "incr".into())
        );
        assert_eq!(req.to_string(), "call.decs.shard.the_void.incr");
    }
}
