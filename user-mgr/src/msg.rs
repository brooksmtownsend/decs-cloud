//! Messaging
//!
//! The user manager should have the following subscriptions
//!    get.decs.users ([GW GET]/api/decs/users)
//!    get.decs.user.* ([GW GET]/api/decs/user/{user-name})
//!    access.decs.user.*
//!    access.decs.users
//!    call.decs.user.*.set (updates a user)
//!    call.decs.users.add (creates a user)
//!

use crate::store;
use decscloud_common as codec;
use decscloud_common::gateway::{ResProtocolRequest, ResourceIdentifier};
use decscloud_common::users::User;
use guest::prelude::*;

/// Examine the subject of the message and invoke the appopriate function
pub fn handle_message(
    ctx: &CapabilitiesContext,
    msg: impl Into<messaging::DeliverMessage>,
) -> CallResult {
    let msg = msg.into().message;
    if let Some(msg) = msg {
        match ResProtocolRequest::from(msg.subject.as_str()) {
            ResProtocolRequest::Get(_) if msg.subject.ends_with("users") => {
                handle_get_collection(ctx, &msg)
            }
            ResProtocolRequest::Add(_) => handle_create(ctx, &msg),
            ResProtocolRequest::Get(rid) => handle_get_single(ctx, &rid, &msg),
            ResProtocolRequest::Set(_) => handle_set(ctx, &msg),
            ResProtocolRequest::Access(_) => handle_access(ctx, &msg),
            _ => Err("unknown service request format".into()),
        }
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
    let user = extract_user_from_set(&msg.body)?;
    ctx.log(&format!(
        "Handling set request: {}, reply-to: {}",
        msg.subject, msg.reply_to
    ));
    update_user(ctx, &user)
}

fn handle_create(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let user = extract_user_from_set(&msg.body)?;
    ctx.log(&format!(
        "Handling new request: {}, reply-to: {}",
        msg.subject, msg.reply_to
    ));
    create_user(ctx, &user, msg)
}

fn create_user(
    ctx: &CapabilitiesContext,
    user: &User,
    msg: &messaging::BrokerMessage,
) -> CallResult {
    match store::create_user(ctx, &user) {
        Ok((pos, resid)) => {
            publish_collection_add(ctx, &user, pos)?;
            if !msg.reply_to.is_empty() {
                let result = json!({ "result": resid });
                ctx.msg()
                    .publish(&msg.reply_to, None, &serde_json::to_vec(&result)?)?;
            }
            Ok(vec![])
        }
        Err(e) => Err(e),
    }
}

fn update_user(ctx: &CapabilitiesContext, user: &User) -> CallResult {
    match store::update_user(ctx, &user) {
        Ok(_) => publish_model_change(ctx, &user),
        Err(e) => Err(e),
    }
}

fn publish_collection_add(ctx: &CapabilitiesContext, user: &User, pos: usize) -> CallResult {
    let subject = "event.decs.users.add";
    let item = format!("decs.user.{}", user.id);
    let rid = ResourceIdentifier { rid: item };
    let out = json!({
        "value": rid,
        "idx": pos
    });
    ctx.msg()
        .publish(&subject, None, &serde_json::to_vec(&out)?)?;
    Ok(vec![])
}

fn publish_model_change(ctx: &CapabilitiesContext, user: &User) -> CallResult {
    let item = format!("decs.user.{}", user.id);
    let subject = format!("event.{}.change", item);

    let out = json!({ "values": user });
    ctx.msg()
        .publish(&subject, None, &serde_json::to_vec(&out)?)?;
    Ok(vec![])
}

fn extract_user_from_set(body: &[u8]) -> Result<User> {
    let v: serde_json::Value = serde_json::from_slice(body)?;
    let user = &v["params"];
    Ok(serde_json::from_value(user.clone())?)
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
    let userlist = store::get_users(ctx)?;

    let rids: Vec<_> = userlist
        .iter()
        .map(|s| codec::gateway::ResourceIdentifier {
            rid: format!("decs.user.{}", s),
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

fn handle_get_single(
    ctx: &CapabilitiesContext,
    rid: &str,
    msg: &messaging::BrokerMessage,
) -> CallResult {
    match store::get_user_details(ctx, rid) {
        Ok(user) => {
            let result = json!({
                "result": {
                    "model": user
                }
            });
            ctx.msg()
                .publish(&msg.reply_to, None, &serde_json::to_vec(&result)?)?;
        }
        Err(ref e) if e.to_string() == store::NOT_FOUND => {
            ctx.msg().publish(
                &msg.reply_to,
                None,
                &serde_json::to_vec(&codec::gateway::error_not_found("No such user"))?,
            )?;
        }
        Err(e) => {
            ctx.msg().publish(
                &msg.reply_to,
                None,
                &serde_json::to_vec(&codec::gateway::error_invalid_params(&format!("{}", e)))?,
            )?;
            ctx.log(&format!("Failed to retrieve user: {}", e));
            return Err(e);
        }
    }
    Ok(vec![])
}
