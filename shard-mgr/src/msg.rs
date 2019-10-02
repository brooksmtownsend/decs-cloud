//! Messaging
//!
//! The shard manager should have the following subscriptions
//!    get.decs.shards ([GW GET]/api/decs/shards)
//!    get.decs.shard.* ([GW GET]/api/decs/shard/{shard-name})
//!    access.decs.shard.*
//!    access.decs.shards
//!

use crate::store;
use decscloud_codec as codec;
use guest::prelude::*;

const GW_ACCESS_PREFIX: &str = "access.decs.shard";
const GW_GET_COLLECTION: &str = "get.decs.shards";
const GW_GET_SINGLE_PREFIX: &str = "get.decs.shard.";

/// Examine the subject of the message and invoke the appopriate function
pub fn handle_message(
    ctx: &CapabilitiesContext,
    msg: impl Into<messaging::DeliverMessage>,
) -> CallResult {
    let msg = msg.into().message;
    if let Some(msg) = msg {
        if msg.subject == GW_GET_COLLECTION {
            handle_get_collection(&ctx, &msg)?;
        } else if msg.subject.starts_with(GW_GET_SINGLE_PREFIX) {
            handle_get_single(&ctx, &msg)?;
        } else if msg.subject.starts_with(GW_ACCESS_PREFIX) {
            handle_access(&ctx, &msg)?;
        }
        Ok(vec![])
    } else {
        Err("no message payload on subject".into())
    }
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

fn handle_get_collection(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    // If we don't have any shards, create "the void" by default
    let shardlist = {
        let l = store::get_shards(ctx)?;
        if l.is_empty() {
            let thevoid = codec::shard::Shard::the_void();
            store::add_shard(ctx, &thevoid)?;
            vec![thevoid.name]
        } else {
            l
        }
    };

    let rids: Vec<_> = shardlist
        .iter()
        .map(|s| codec::gateway::ResourceIdentifier {
            rid: format!("decs.shard.{}", s),
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

fn handle_get_single(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let tokens: Vec<&str> = msg.subject.split('.').collect();
    if tokens.len() != 4 {
        // get.decs.shard.xxx
        Err("incorrectly formatted single-shard get request".into())
    } else {
        let shard = store::get_shard_details(ctx, tokens[3])?;
        let result = json!({
            "result": {
                "model": {
                    "name": shard.name,
                    "current": shard.current,
                    "capacity": shard.capacity
                }
            }
        });
        ctx.msg()
            .publish(&msg.reply_to, None, &serde_json::to_vec(&result)?)?;
        Ok(vec![])
    }
}
