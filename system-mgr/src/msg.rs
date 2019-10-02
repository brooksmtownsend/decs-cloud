//! Messaging
//!
//! The system manager should have the following subscriptions
//!    get.decs.systems [GW GET]/api/decs/systems
//!    access.decs.systems
//!    get.decs.system.* [GW GET]/api/decs/system/{system-name}
//!    decs.system.registry.replies
//!    decs.{shard}.gameloop
//!

use crate::store;
use decscloud_codec as codec;
use decscloud_codec::gateway::ResourceIdentifier;
use decscloud_codec::systemmgr::System;
use guest::prelude::*;

const PING_EVERY_TICKS: i64 = 200;
const REGISTRY_PING_SUBJECT: &str = "decs.system.registry";
const REGISTRY_PONG_SUBJECT: &str = "decs.system.registry.replies";

const GAMELOOP_SUFFIX: &str = ".gameloop";
const GW_GET_PREFIX: &str = "get.decs.system";
const GW_GET_COLLECTION: &str = "get.decs.systems";
const GW_ACCESS_PREFIX: &str = "access.decs.system";
const GW_GET_SINGLE_PREFIX: &str = "get.decs.system.";

pub fn handle_timer(
    ctx: &CapabilitiesContext,
    tick: impl Into<codec::timer::TimerTick>,
) -> CallResult {
    let tick = tick.into();
    if tick.seq_no % PING_EVERY_TICKS == 0 {
        emit_ping(ctx)
    } else {
        Ok(vec![])
    }
}

fn emit_ping(ctx: &CapabilitiesContext) -> CallResult {
    ctx.log("Emitting registration ping");
    ctx.msg()
        .publish(REGISTRY_PING_SUBJECT, Some(REGISTRY_PONG_SUBJECT), &[])?;
    Ok(vec![])
}

/// Examine the subject of the message and invoke the appopriate function
pub fn handle_message(
    ctx: &CapabilitiesContext,
    msg: impl Into<messaging::DeliverMessage>,
) -> CallResult {
    let msg = msg.into().message;
    if let Some(msg) = msg {
        if msg.subject == REGISTRY_PONG_SUBJECT {
            handle_registration(&ctx, &msg)?;
        } else if msg.subject.starts_with(GW_GET_PREFIX) {
            handle_get(&ctx, &msg)?;
        } else if msg.subject.starts_with(GW_ACCESS_PREFIX) {
            handle_access(&ctx, &msg)?;
        } else if msg.subject.ends_with(GAMELOOP_SUFFIX) {
            handle_gameloop(&ctx, &msg)?;
        }
        Ok(vec![])
    } else {
        Err("no message payload on subject".into())
    }
}

// Upon receipt of a game loop tick, the system manager must
//   for each discovered system:
//     determine if it is the right time to emit a message for the given system (based on system FPS desire)
//     emit an entity frame message for each entity that has all of that system's components
//
// NOTE: it is the responsibility of the implementing system (e.g. physics, combat, nav, radar, etc) to
// query the values of any components it needs when it gets an entity frame
fn handle_gameloop(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let gtick: codec::timer::GameLoopTick = serde_json::from_slice(&msg.body)?;
    let shard = gtick.shard;
    let systems = store::get_systems(ctx)?;
    let systemlist = store::get_system_list(ctx, systems)?;

    for system in systemlist.iter() {
        if !should_publish(system.framerate, gtick.elapsed_ms, gtick.seq_no) {
            continue;
        }
        let entities =
            store::get_entities_for_component_set(ctx, &shard, system.components.as_slice())?;
        for entity in entities.iter() {
            let cf = codec::systemmgr::EntityFrame {
                seq_no: gtick.seq_no,
                elapsed_ms: gtick.elapsed_ms * system_modulus(system.framerate, gtick.elapsed_ms),
                shard: shard.to_string(),
                entity_id: entity.to_string(),
            };
            let subject = format!("decs.frames.{}.{}", shard, system.name);
            ctx.msg()
                .publish(&subject, None, &serde_json::to_vec(&cf)?)?;
        }
    }

    Ok(vec![])
}

fn handle_registration(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let system: codec::systemmgr::System = serde_json::from_slice(&msg.body)?;
    let existed = store::put_system(ctx, &system)?;
    if !existed {
        publish_collection_add(ctx, &system)?;
    } else {
        publish_model_change(ctx, &system)?;
    }
    Ok(vec![])
}

fn handle_access(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let result = json!({
        "result" : {
            "get" : true
        }
    });
    ctx.msg()
        .publish(&msg.reply_to, None, &serde_json::to_vec(&result)?)?;
    Ok(vec![])
}

fn handle_get(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    if msg.subject == GW_GET_COLLECTION {
        get_collection(ctx, msg)
    } else if msg.subject.starts_with(GW_GET_SINGLE_PREFIX) {
        get_single(ctx, msg)
    } else {
        Ok(vec![])
    }
}

fn get_collection(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let syslist = store::get_systems(ctx)?;
    let rids: Vec<_> = syslist
        .iter()
        .map(|s| codec::gateway::ResourceIdentifier {
            rid: format!("decs.system.{}", s),
        })
        .collect();
    let result = json!({
        "result": {
            "collection": rids
        }
    });
    ctx.msg()
        .publish(&msg.reply_to, None, &serde_json::to_vec(&result)?)?;
    Ok(vec![])
}

fn get_single(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let tokens: Vec<&str> = msg.subject.split('.').collect();
    if tokens.len() != 4 {
        // get.decs.system.xxx
        Err("incorrectly formatted single-system get request".into())
    } else {
        let s_name = tokens[3];
        let system = store::get_system_details(ctx, s_name)?;
        let result = json!({
            "result": {
                "model": {
                    "components" : system.components.join(","), // resgate only allows primitives or RIDs
                    "framerate": system.framerate,
                    "name": system.name,
                }
            }
        });
        ctx.msg()
            .publish(&msg.reply_to, None, &serde_json::to_vec(&result)?)?;
        Ok(vec![])
    }
}

fn system_modulus(framerate: u32, elapsed_ms: u32) -> u32 {
    (1000.0 / elapsed_ms as f32 / framerate as f32) as u32
}

fn should_publish(framerate: u32, elapsed_ms: u32, seq_no: u64) -> bool {
    seq_no % u64::from(system_modulus(framerate, elapsed_ms)) == 0
}

fn publish_collection_add(ctx: &CapabilitiesContext, system: &System) -> Result<()> {
    let subject = "event.decs.systems.add";
    let item = format!("decs.system.{}", system.name);

    let rid = ResourceIdentifier { rid: item };
    let out = json!({
        "value" : rid,
        "idx": 0
    });
    ctx.log(&format!("Publishing Collection Add, subject: {}", subject));
    ctx.msg()
        .publish(&subject, None, &serde_json::to_vec(&out)?)?;
    Ok(())
}

fn publish_model_change(ctx: &CapabilitiesContext, system: &System) -> Result<()> {
    let item = format!("decs.system.{}", system.name);
    let subject = format!("event.{}.change", item);

    let out = json!({ "values": system });
    ctx.log(&format!("Publishing Model Change, subject: {}", subject));
    ctx.msg()
        .publish(&subject, None, &serde_json::to_vec(&out)?)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::system_modulus;

    #[test]
    fn test_modulus() {
        // At 1 FPS, with an elapsed of 100ms , modulus should be 10 (how many elapseds in the FPS)
        assert_eq!(10, system_modulus(1, 100));
        assert_eq!(1, system_modulus(10, 100));
        assert_eq!(1, system_modulus(1, 1000));
    }
}
