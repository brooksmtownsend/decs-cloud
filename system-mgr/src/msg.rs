//! Messaging
//!
//! The system manager should have the following subscriptions
//!    get.decs.systems [GW GET]/api/decs/systems
//!    access.decs.systems
//!    get.decs.system.* [GW GET]/api/decs/system/{system-name}
//!    decs.system.registry.replies
//!

use crate::store;
use decscloud_codec as codec;
use guest::prelude::*;

const PING_EVERY_TICKS: i64 = 200;
const REGISTRY_PING_SUBJECT: &str = "decs.system.registry";
const REGISTRY_PONG_SUBJECT: &str = "decs.system.registry.replies";

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
        // publish_frame(...)
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
        }
        Ok(vec![])
    } else {
        Err("no message payload on subject".into())
    }
}

fn handle_registration(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let system: codec::systemmgr::System = serde_json::from_slice(&msg.body)?;
    store::put_system(ctx, &system)?;

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
    if tokens.len() != 4 { // get.decs.system.xxx
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
