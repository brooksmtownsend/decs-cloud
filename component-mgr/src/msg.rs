use crate::store;
use codec::gateway::ResProtocolRequest;
use decscloud_common as codec;
use guest::prelude::*;

// get.decs.components.{shard-id}.{entity-id}.{component-name}
// call.decs.components.{shard-id}.{entity-id}.{component-name}.set (model)
// call.decs.components.{shard-id}.{entity-id}.{component-name}.new (collection)
// call.decs.components.{shard-id}.{entity-id}.{component-name}.delete (collection or model)
// access.decs.components.>
pub(crate) fn handle_message(
    ctx: &CapabilitiesContext,
    msg: impl Into<messaging::DeliverMessage>,
) -> CallResult {
    let msg = msg.into().message;

    if let Some(msg) = msg {
        match ResProtocolRequest::from(msg.subject.as_str()) {
            ResProtocolRequest::Access(ref refid) => handle_access(ctx, &msg, refid),
            ResProtocolRequest::Get(ref refid) => handle_get(ctx, &msg, refid),
            ResProtocolRequest::Set(ref refid) => handle_model_set(ctx, &msg, refid),
            ResProtocolRequest::New(ref refid) => handle_collection_new(ctx, &msg, refid),
            ResProtocolRequest::Delete(ref refid) => handle_delete(ctx, &msg, refid),
            _ => Err("unknown service request".into()),
        }
    } else {
        Err("no message payload on subject".into())
    }
}

/// Responds to a RES messaging server with the access metadata
/// By default, we allow all access
/// TODO: upgrade access determination
fn handle_access(
    ctx: &CapabilitiesContext,
    msg: &messaging::BrokerMessage,
    _rid: &str,
) -> CallResult {
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

/// Responds to a RES protocol GET request, which can be for a single model
/// or a collection (which is an array of rids)
fn handle_get(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage, rid: &str) -> CallResult {
    ctx.log(&format!(
        "Handling GET request: {}, rid: {}",
        msg.subject, rid
    ));

    if let store::ComponentType::Model = store::component_type(ctx, rid)? {
        handle_single_get(ctx, msg, rid)
    } else {
        handle_collection_get(ctx, msg, rid)
    }
}

fn handle_delete(
    ctx: &CapabilitiesContext,
    msg: &messaging::BrokerMessage,
    rid: &str,
) -> CallResult {
    ctx.log(&format!(
        "Handling DELETE request: {}, rid: {}",
        msg.subject, rid
    ));
    if let store::ComponentType::Model = store::component_type(ctx, rid)? {
        handle_model_delete(ctx, msg, rid)
    } else {
        handle_collection_delete_item(ctx, msg, rid)
    }
}

fn handle_model_delete(
    ctx: &CapabilitiesContext,
    msg: &messaging::BrokerMessage,
    rid: &str,
) -> CallResult {
    store::delete_component(ctx, rid)?;

    // TODO: figure out what to publish for RES protocol model deletion

    let tokens: Vec<&str> = rid.split('.').collect();
    let shard = tokens[2]; // decs.components.(shard).(entity)...
    publish_update_shard(ctx, shard, -1)?;

    if !msg.reply_to.is_empty() {
        ctx.msg().publish(
            &msg.reply_to,
            None,
            &serde_json::to_vec(&codec::gateway::success_response())?,
        )?;
    };

    Ok(vec![])
}

fn handle_collection_delete_item(
    ctx: &CapabilitiesContext,
    msg: &messaging::BrokerMessage,
    rid: &str,
) -> CallResult {
    let raw: serde_json::Value = serde_json::from_slice(&msg.body)?;
    let item_rid: String = raw["params"]["rid"].as_str().unwrap().to_string();
    let idx = store::remove_component_from_collection(ctx, rid, &item_rid)?;
    publish_collection_remove(ctx, rid, idx)?;
    if !msg.reply_to.is_empty() {
        ctx.msg().publish(
            &msg.reply_to,
            None,
            &serde_json::to_vec(&codec::gateway::success_response())?,
        )?;
    };

    Ok(vec![])
}

/// Responds to a RES protocol GET request with a collection of reference IDs
fn handle_collection_get(
    ctx: &CapabilitiesContext,
    msg: &messaging::BrokerMessage,
    rid: &str,
) -> CallResult {
    let rids = store::get_collection_rids(ctx, &rid)?;

    let refs: Vec<_> = rids
        .iter()
        .map(|s| codec::gateway::ResourceIdentifier { rid: s.clone() })
        .collect();
    let result = json!({
        "result": {
            "collection": refs
        }
    });
    ctx.msg()
        .publish(&msg.reply_to, None, &serde_json::to_vec(&result)?)?;
    Ok(vec![])
}

fn handle_single_get(
    ctx: &CapabilitiesContext,
    msg: &messaging::BrokerMessage,
    rid: &str,
) -> CallResult {
    match store::get_component(ctx, rid) {
        Ok(c) => {
            let model_json: serde_json::Value = serde_json::from_str(&c)?;
            ctx.msg().publish(
                &msg.reply_to,
                None,
                &serde_json::to_vec(&codec::gateway::model_result(model_json))?,
            )?;
            Ok(vec![])
        }
        Err(ref e) if e.to_string() == store::NO_SUCH_COMPONENT => {
            ctx.msg().publish(
                &msg.reply_to,
                None,
                &serde_json::to_vec(&codec::gateway::error_not_found("No such component"))?,
            )?;
            Ok(vec![])
        }
        Err(e) => {
            ctx.log(&format!("Failed to retrieve component: {}", e));
            Err(e)
        }
    }
}

fn handle_collection_new(
    ctx: &CapabilitiesContext,
    msg: &messaging::BrokerMessage,
    rid: &str,
) -> CallResult {
    let new_component = extract_model_from_set(&msg.body)?;
    ctx.log(&format!(
        "Handling collection new: {}, rid: {}",
        msg.subject, rid
    ));
    let (new_index, item_rid) =
        store::add_component_to_collection(ctx, rid, &serde_json::to_string(&new_component)?)?;
    publish_collection_add(ctx, &rid, &item_rid, new_index)?;
    let resid = codec::gateway::ResourceIdentifier {
        rid: item_rid.clone(),
    };
    let result = json!({ "result": resid });
    if !msg.reply_to.is_empty() {
        ctx.msg()
            .publish(&msg.reply_to, None, &serde_json::to_vec(&result)?)?;
    };
    Ok(vec![])
}

/// When the RES protocol invokes a set for a single model, the payload looks as follows:
/// ```
/// {
///   "params" : ... the model object ..,
///   "token" : .. access token ...,
///   "cid" : ... connection id ...
/// }
/// ```
fn handle_model_set(
    ctx: &CapabilitiesContext,
    msg: &messaging::BrokerMessage,
    rid: &str,
) -> CallResult {
    let comp = extract_model_from_set(&msg.body)?;
    ctx.log(&format!(
        "Handling set request: {}, reply-to: {}",
        msg.subject, msg.reply_to
    ));
    let existed = store::put_component(ctx, rid, &serde_json::to_string(&comp)?)?;
    if !existed {
        let shard = shard_from_rid(rid);
        publish_update_shard(ctx, &shard, 1)?;
    }
    publish_model_change(ctx, comp, rid)?;
    if !msg.reply_to.is_empty() {
        ctx.msg().publish(
            &msg.reply_to,
            None,
            &serde_json::to_vec(&codec::gateway::success_response())?,
        )?;
    };
    Ok(vec![])
}

fn publish_update_shard(ctx: &CapabilitiesContext, shard: &str, amount: i32) -> Result<()> {
    let out = json!({
        "params": {
            "amount": amount
        }
    });
    let subject = format!("call.decs.shard.{}.incr", shard);
    ctx.msg()
        .publish(&subject, None, &serde_json::to_vec(&out)?)?;
    Ok(())
}

fn publish_collection_add(
    ctx: &CapabilitiesContext,
    rid: &str,
    item_rid: &str,
    idx: usize,
) -> Result<()> {
    let subject = format!("event.{}.add", rid);
    let shard = shard_from_rid(rid);

    let resid = decs::gateway::ResourceIdentifier {
        rid: item_rid.to_string(),
    };
    let out = json!({
        "value" : resid,
        "idx": idx
    });
    ctx.log(&format!("Publishing Collection Add, subject: {}", subject));
    ctx.msg()
        .publish(&subject, None, &serde_json::to_vec(&out)?)?;
    publish_update_shard(ctx, &shard, 1)?; // increment component count by 1
    Ok(())
}

fn publish_collection_remove(ctx: &CapabilitiesContext, rid: &str, idx: usize) -> Result<()> {
    let subject = format!("event.{}.remove", rid);
    let shard = shard_from_rid(rid);
    let out = json!({ "idx": idx });
    ctx.log(&format!(
        "Publishing collection remove, subject: {}, idx: {}",
        subject, idx,
    ));
    ctx.msg()
        .publish(&subject, None, &serde_json::to_vec(&out)?)?;
    publish_update_shard(ctx, &shard, -1)?; // decrement component count by 1
    Ok(())
}

fn publish_model_change(
    ctx: &CapabilitiesContext,
    comp: serde_json::Value,
    rid: &str,
) -> Result<()> {
    let subject = format!("event.{}.change", rid);

    let out = json!({ "values": comp });
    ctx.log(&format!("Publishing Model Change, subject: {}", subject));
    ctx.msg()
        .publish(&subject, None, &serde_json::to_vec(&out)?)?;
    Ok(())
}

fn extract_model_from_set(body: &[u8]) -> Result<serde_json::Value> {
    let v: serde_json::Value = serde_json::from_slice(body)?;
    let comp = &v["params"];
    Ok(comp.clone())
}

// decs.components.(shard).(entity).x.y
fn shard_from_rid(rid: &str) -> String {
    let tokens: Vec<&str> = rid.split('.').collect();
    tokens[2].to_string()
}

#[cfg(test)]
mod test {
    #[test]
    fn test_extract_from_set() {
        let set_payload = br#"
        {
            "params": {
                "mag": 20
            },
            "token": null,
            "cid": "bm3a0mhcl5cuoia4aju0"
        }
        "#;

        let val = super::extract_model_from_set(set_payload).unwrap();
        assert_eq!(val["mag"], 20);
    }
}
