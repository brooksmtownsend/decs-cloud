use crate::store;
use decscloud_codec as codec;
use guest::prelude::*;

const GET_COMPONENT_PREFIX: &str = "get.decs.components.";
const SET_COMPONENT_PREFIX: &str = "call.decs.components.";
const GW_ACCESS_PREFIX: &str = "access.decs.components.";

// get.decs.components.{shard-id}.{entity-id}.{component-name}
// call.decs.components.{shard-id}.{entity-id}.{component-name}.set
// access.decs.components.>
pub(crate) fn handle_message(
    ctx: &CapabilitiesContext,
    msg: impl Into<messaging::DeliverMessage>,
) -> CallResult {
    let msg = msg.into().message;

    if let Some(msg) = msg {
        if msg.subject.starts_with(GET_COMPONENT_PREFIX) {
            handle_get(&ctx, &msg)
        } else if msg.subject.starts_with(SET_COMPONENT_PREFIX) &&
                  msg.subject.ends_with("set") {
            handle_set(&ctx, &msg)
        } else if msg.subject.starts_with(GW_ACCESS_PREFIX) {
            handle_access(&ctx, &msg)
        } else {
            Err("unrecognized component management subject".into())
        }
    } else {
        Err("no message payload on subject".into())
    }
}

fn handle_access(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let result = json!({
        "result" : {
            "get" : true,
            "call" : "set"
        }
    });
    ctx.msg()
        .publish(&msg.reply_to, None, &serde_json::to_vec(&result)?)?;
    Ok(vec![])
}

fn handle_get(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let tokens: Vec<&str> = msg.subject.split('.').collect();
    if tokens.len() == 6 {
        handle_single_get(ctx, msg, &tokens)
    } else {
        Err("invalid subject format for component management".into())
    }
}

fn handle_single_get(
    ctx: &CapabilitiesContext,
    msg: &messaging::BrokerMessage,
    tokens: &[&str],
) -> CallResult {
    match store::get_component(ctx, tokens) {
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

/// When the RES protocol invokes a set, the payload looks as follows: 
/// ```
/// {
///   "params" : ... the model object ..,
///   "token" : .. access token ...,
///   "cid" : ... connection id ...
/// }
/// ```
fn handle_set(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let tokens: Vec<&str> = msg.subject.split('.').collect();
    let comp = extract_model_from_set(&msg.body)?;
    
    store::put_component(ctx, &tokens, &serde_json::to_string(&comp)?)?;
    ctx.msg().publish(&msg.reply_to, None,
        &serde_json::to_vec(&codec::gateway::success_response())?)?;
    Ok(vec![])
}

fn extract_model_from_set(body: &[u8]) -> Result<serde_json::Value> {
    let v: serde_json::Value = serde_json::from_slice(body)?;
    let comp = &v["params"];
    Ok(comp.clone())
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

/*
/// Handles messages published on the topic `component.[shard].[entity].[name].[action]`
fn handle_msg(ctx: &CapabilitiesContext, msg: &messaging::BrokerMessage) -> CallResult {
    let tokens: Vec<&str> = msg.subject.split('.').collect();
    if tokens.len() != 5 && tokens[0] != "component" {
        Err("poorly formed subject".into())
    } else {
        let (shard, entity, compname, op) = (tokens[1], tokens[2], tokens[3], tokens[4]);
        let key = format!("comp:{}:{}:{}", shard, entity, compname);
        let val = std::str::from_utf8(&msg.body)?;
        match op {
            "put" => ctx.kv().set(&key, val, None)?,
            "del" => ctx.kv().del_key(&key)?,
            _ => return Err("Invalid component operation".into()),
        };

        Ok(vec![])
    }
}*/
