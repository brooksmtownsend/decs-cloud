//! Messaging
//!
//! The shard manager should have the following subscriptions
//!    get.decs.shards ([GW GET]/api/decs/shards)
//!    get.decs.shard.* ([GW GET]/api/decs/shard/{shard-name})
//!    access.decs.shard.*
//!    access.decs.shards
//!    call.decs.shards.*.set (sets/creates a shard)
//!

use crate::store;
use decscloud_codec as codec;
use decscloud_codec::gateway::ResourceIdentifier;
use decscloud_codec::shard::Shard;
use guest::prelude::*;

const GW_SET_PREFIX: &str = "call.decs.shard.";
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
        } else if msg.subject.starts_with(GW_SET_PREFIX) && msg.subject.ends_with("set") {
            handle_set(&ctx, &msg)?;
        }
        Ok(vec![])
    } else {
        Err("no message payload on subject".into())
    }
}

/// When the RES protocol invokes a set, the payload looks as follows:
/// ```
/// {
///   "params" : ... the model object ..,
///   "token" : .. access token ...,
///   "cid" : ... connection id ...
/// }
/// ```
fn handle_set(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let shard = extract_shard_from_set(&msg.body)?;
    ctx.log(&format!(
        "Handling set request: {}, reply-to: {}",
        msg.subject, msg.reply_to
    ));
    set_shard(ctx, &shard)
}

fn set_shard(ctx: &CapabilitiesContext, shard: &Shard) -> CallResult {
    match store::put_shard(ctx, shard) {
        Ok((pos, existed)) => {
            if !existed {
                publish_collection_add(ctx, shard, pos)
            } else {
                publish_model_change(ctx, shard)
            }
        }
        Err(e) => Err(e),
    }
}

fn publish_collection_add(ctx: &CapabilitiesContext, shard: &Shard, pos: usize) -> CallResult {
    let subject = "event.decs.shards.add";
    let item = format!("decs.shard.{}", shard.name);
    let rid = ResourceIdentifier { rid: item };
    let out = json!({
        "value": rid,
        "idx": pos
    });
    ctx.msg()
        .publish(&subject, None, &serde_json::to_vec(&out)?)?;
    Ok(vec![])
}

fn publish_model_change(ctx: &CapabilitiesContext, shard: &Shard) -> CallResult {
    let item = format!("decs.shard.{}", shard.name);
    let subject = format!("event.{}.change", item);

    let out = json!({ "values": shard });
    ctx.msg()
        .publish(&subject, None, &serde_json::to_vec(&out)?)?;
    Ok(vec![])
}

fn extract_shard_from_set(body: &[u8]) -> Result<Shard> {
    let v: serde_json::Value = serde_json::from_slice(body)?;
    let shard = &v["params"];
    Ok(serde_json::from_value(shard.clone())?)
}

fn handle_access(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let result = json!({
        "result" : {
            "get" : true,
            "call" : "*"
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
            set_shard(ctx, &thevoid)?;
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
        match store::get_shard_details(ctx, tokens[3]) {
            Ok(shard) => {
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
            }
            Err(ref e) if e.to_string() == store::NOT_FOUND => {
                ctx.msg().publish(
                    &msg.reply_to,
                    None,
                    &serde_json::to_vec(&codec::gateway::error_not_found("No such component"))?,
                )?;
            }
            Err(e) => {
                ctx.log(&format!("Failed to retrieve component: {}", e));
                return Err(e);
            }
        }
        Ok(vec![])
    }
}
