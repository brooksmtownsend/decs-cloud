// Copyright 2015-2019 Capital One Services, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// The component manager performs the following tasks
/// 1. Listen for an incoming delivered message from a wascap:messaging subscription
/// 2. If that message is on the subject component.[shard].[entity].[name].[action] the component manager
/// will then
/// a) `put` the raw JSON value into the K/V store
/// b) `get` the raw JSON value from the K/V store (will reply on the reply-to subject)
/// c) `del`ete the raw JSON value from the K/V store
///
/// For example, if a message is published to `component.abc1234.cbfjh3412.location.put` with the body:
/// ```
/// {
///     "x": 12,
///     "y": 30,
///     "z": 800
/// }
/// ```
/// Then the K/V store will place that value in the key `comp:abc1234:cbfjh3412:location`, and this value
/// represents the current location of the given entity within the given space. This value very likely
/// came from a system, e.g. `physics` or `movement` that declared an interest in the `velocity` component,
/// received a tick with a `velocity`, and updated the location accordingly.
extern crate wascap_guest as guest;

use guest::prelude::*;

call_handler!(handle_call);

pub fn handle_call(ctx: &CapabilitiesContext, operation: &str, msg: &[u8]) -> CallResult {
    match operation {
        messaging::OP_DELIVER_MESSAGE => handle_message(ctx, msg),
        core::OP_HEALTH_REQUEST => Ok(vec![]),
        _ => Err("bad dispatch".into()),
    }
}

fn handle_message(
    ctx: &CapabilitiesContext,
    msg: impl Into<messaging::DeliverMessage>,
) -> CallResult {
    let msg = msg.into();

    match msg.message {
        Some(ref m) => handle_msg(ctx, m),
        None => Err("no message delivered".into()),
    }
}

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
}
